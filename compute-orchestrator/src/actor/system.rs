use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use super::message::{ActorMessage, ActorState};
use super::pid::ProcessId;

pub type ActorHandler = Arc<
    dyn Fn(ActorMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

pub struct ActorContext {
    pub pid: ProcessId,
    pub state: ActorState,
    pub handler: ActorHandler,
}

#[derive(Debug, Clone, Copy)]
pub enum SupervisionStrategy {
    OneForOne,
    AllForOne,
}

pub struct ActorSystem {
    node_id: u32,
    actors: Arc<Mutex<HashMap<u64, mpsc::Sender<ActorMessage>>>>,
    states: Arc<Mutex<HashMap<u64, ActorState>>>,
    next_id: AtomicU64,
    supervision: SupervisionStrategy,
    max_restarts: u32,
}

impl ActorSystem {
    pub fn new(node_id: u32, supervision: SupervisionStrategy, max_restarts: u32) -> Self {
        Self {
            node_id,
            actors: Arc::new(Mutex::new(HashMap::new())),
            states: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicU64::new(1),
            supervision,
            max_restarts,
        }
    }

    pub async fn spawn<F, Fut>(&self, handler_fn: F, mailbox_size: usize) -> ProcessId
    where
        F: Fn(ActorMessage) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let bounded_mailbox_size = if mailbox_size == 0 {
            1
        } else if mailbox_size > 65535 {
            65535
        } else {
            mailbox_size
        };

        let actor_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let pid = ProcessId::new(self.node_id, actor_id);
        let (tx, mut rx) = mpsc::channel::<ActorMessage>(bounded_mailbox_size);

        self.actors.lock().await.insert(actor_id, tx);
        self.states
            .lock()
            .await
            .insert(actor_id, ActorState::Starting);

        let handler = Arc::new(handler_fn);
        let actors = self.actors.clone();
        let states = self.states.clone();
        let max_restarts = self.max_restarts;
        let supervision = self.supervision;
        let node_id = self.node_id;

        tokio::spawn(async move {
            info!("Actor {}:{} started", node_id, actor_id);

            let mut restart_count = 0u32;

            loop {
                let result: std::result::Result<(), ()> = tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(m) => {
                                handler(m).await;
                                Ok(())
                            }
                            None => {
                                warn!("Actor {}:{} mailbox closed", node_id, actor_id);
                                break;
                            }
                        }
                    }
                };

                if let Err(_e) = result {
                    error!("Actor {}:{} encountered error", node_id, actor_id);
                    restart_count += 1;

                    if restart_count > max_restarts {
                        error!(
                            "Actor {}:{} exceeded max restarts ({}), stopping",
                            node_id, actor_id, max_restarts
                        );
                        let mut st = states.lock().await;
                        st.insert(actor_id, ActorState::Failed);
                        break;
                    }

                    warn!(
                        "Actor {}:{} restarting (attempt {}/{})",
                        node_id, actor_id, restart_count, max_restarts
                    );

                    match supervision {
                        SupervisionStrategy::OneForOne => {
                            let mut st = states.lock().await;
                            st.insert(actor_id, ActorState::Starting);
                        }
                        SupervisionStrategy::AllForOne => {
                            let mut st = states.lock().await;
                            for (id, state) in st.iter_mut() {
                                if *id != actor_id {
                                    *state = ActorState::Starting;
                                }
                            }
                            st.insert(actor_id, ActorState::Starting);
                        }
                    }
                } else {
                    debug!("Actor {}:{} message processed", node_id, actor_id);
                }
            }

            let mut actors = actors.lock().await;
            actors.remove(&actor_id);

            let mut st = states.lock().await;
            st.insert(actor_id, ActorState::Stopped);

            info!("Actor {}:{} stopped", node_id, actor_id);
        });

        pid
    }

    pub async fn send(&self, message: ActorMessage) -> crate::error::Result<()> {
        let actor_id = message.recipient.actor_id;
        let actors = self.actors.lock().await;
        let sender = actors
            .get(&actor_id)
            .ok_or(crate::error::OrchestratorError::ActorNotFound(actor_id))?;

        sender.send(message).await.map_err(|e| {
            crate::error::OrchestratorError::Network(format!(
                "mailbox closed for actor {}: {}",
                actor_id, e
            ))
        })?;

        Ok(())
    }

    pub async fn get_state(&self, actor_id: u64) -> Option<ActorState> {
        let states = self.states.lock().await;
        states.get(&actor_id).copied()
    }

    pub async fn list_actors(&self) -> Vec<(u64, ActorState)> {
        let states = self.states.lock().await;
        states.iter().map(|(k, v)| (*k, *v)).collect()
    }

    pub async fn actor_count(&self) -> usize {
        self.actors.lock().await.len()
    }

    pub fn node_id(&self) -> u32 {
        self.node_id
    }
}
