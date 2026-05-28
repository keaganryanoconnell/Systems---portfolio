use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use compute_orchestrator::{
    network::serializer::MessageType, split_workload, ActorMessage, ActorSystem, MacroTask,
    OrchestratorMetrics, ProcessId, RangeSpec, SupervisionStrategy, SwimConfig, SwimNode,
};

#[tokio::test]
async fn test_gossip_startup() {
    let addr1: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let swim = SwimNode::new(
        addr1,
        SwimConfig {
            ping_interval: Duration::from_millis(200),
            ..SwimConfig::default()
        },
    )
    .await
    .unwrap();

    swim.start().await;

    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_actor_spawn_and_send() {
    let system = ActorSystem::new(0, SupervisionStrategy::OneForOne, 3);
    let metrics = Arc::new(OrchestratorMetrics::new());
    let m = metrics.clone();

    let pid = system
        .spawn(
            move |msg| {
                let metrics = m.clone();
                Box::pin(async move {
                    metrics
                        .messages_received
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let _ = msg;
                })
            },
            64,
        )
        .await;

    assert_eq!(system.actor_count().await, 1);

    let msg = ActorMessage::new(
        ProcessId::new(0, 0),
        pid,
        MessageType::TaskDispatch,
        &"hello",
    )
    .unwrap();

    system.send(msg).await.unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(metrics.snapshot().messages_received, 1);
}

#[tokio::test]
async fn test_workload_split_and_dispatch() {
    let macro_task = MacroTask {
        id: 1,
        name: "test".into(),
        payload_type: "computation".into(),
        data_range: RangeSpec { start: 0, end: 100 },
        partition_count: 10,
    };

    let micros = split_workload(&macro_task);
    assert_eq!(micros.len(), 10);
    assert_eq!(micros[0].range.start, 0);
    assert_eq!(micros[0].range.end, 10);
    assert_eq!(micros[9].range.start, 90);
    assert_eq!(micros[9].range.end, 100);

    let system = ActorSystem::new(0, SupervisionStrategy::OneForOne, 3);
    let metrics = Arc::new(OrchestratorMetrics::new());

    for task in micros {
        let m = metrics.clone();
        let pid = system
            .spawn(
                move |_msg| {
                    let met = m.clone();
                    Box::pin(async move {
                        met.tasks_completed
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    })
                },
                64,
            )
            .await;

        let msg =
            ActorMessage::new(ProcessId::new(0, 0), pid, MessageType::TaskDispatch, &task).unwrap();

        system.send(msg).await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    assert_eq!(metrics.snapshot().tasks_completed, 10);
}
