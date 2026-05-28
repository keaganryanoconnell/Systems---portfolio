use std::net::SocketAddr;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use compute_orchestrator::{
    init_tracer, network::serializer::MessageType, split_workload, ActorMessage, ActorSystem,
    MacroTask, OrchestratorMetrics, PeerMetadata, ProcessId, RangeSpec, SupervisionStrategy,
    SwimConfig, SwimNode,
};

#[derive(Parser)]
#[command(name = "compute-orchestrator")]
#[command(version = "0.1.0")]
#[command(about = "Cloud-native distributed compute orchestration layer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Leader {
        #[arg(long, default_value = "0.0.0.0:9000")]
        bind: String,

        #[arg(long)]
        peers: Vec<String>,

        #[arg(long)]
        otlp_endpoint: Option<String>,
    },
    Worker {
        #[arg(long, default_value = "0.0.0.0:9001")]
        bind: String,

        #[arg(long)]
        leader: String,

        #[arg(long)]
        otlp_endpoint: Option<String>,
    },
    Standalone {
        #[arg(long, default_value = "0.0.0.0:9000")]
        bind: String,

        #[arg(long)]
        task_name: String,

        #[arg(long, default_value = "100")]
        task_count: u64,

        #[arg(long, default_value = "10")]
        partitions: u32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("failed to set tracing subscriber: {}", e))?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Leader {
            bind,
            peers,
            otlp_endpoint,
        } => {
            run_leader(&bind, &peers, otlp_endpoint.as_deref()).await?;
        }
        Commands::Worker {
            bind,
            leader,
            otlp_endpoint,
        } => {
            run_worker(&bind, &leader, otlp_endpoint.as_deref()).await?;
        }
        Commands::Standalone {
            bind,
            task_name,
            task_count,
            partitions,
        } => {
            run_standalone(&bind, &task_name, task_count, partitions).await?;
        }
    }

    Ok(())
}

async fn run_leader(
    bind_addr: &str,
    peer_addrs: &[String],
    otlp_endpoint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    init_tracer("compute-orchestrator-leader", otlp_endpoint);

    let bind: SocketAddr = bind_addr
        .parse()
        .map_err(|e| format!("invalid bind address: {}", e))?;
    let swim = SwimNode::new(bind, SwimConfig::default())
        .await
        .map_err(|e| format!("failed to start SWIM node: {}", e))?;

    let peer_sockets: Vec<SocketAddr> = peer_addrs.iter().filter_map(|p| p.parse().ok()).collect();

    swim.join(&peer_sockets).await;
    swim.start().await;

    let system = ActorSystem::new(0, SupervisionStrategy::OneForOne, 3);
    let metrics = Arc::new(OrchestratorMetrics::new());

    let metrics_clone = metrics.clone();
    system
        .spawn(
            move |msg| {
                let m = metrics_clone.clone();
                Box::pin(async move {
                    m.messages_received
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    info!("Leader received: {:?} from {}", msg.msg_type, msg.sender);
                })
            },
            128,
        )
        .await;

    info!("Leader node running on {}. Peers: {:?}", bind, peer_sockets);
    info!("Waiting for tasks... (press Ctrl+C to stop)");

    tokio::signal::ctrl_c().await.ok();
    info!("Leader shutting down");
    Ok(())
}

async fn run_worker(
    bind_addr: &str,
    leader_addr: &str,
    otlp_endpoint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    init_tracer("compute-orchestrator-worker", otlp_endpoint);

    let bind: SocketAddr = bind_addr
        .parse()
        .map_err(|e| format!("invalid bind address: {}", e))?;
    let leader: SocketAddr = leader_addr
        .parse()
        .map_err(|e| format!("invalid leader address: {}", e))?;

    let swim = SwimNode::new(bind, SwimConfig::default())
        .await
        .map_err(|e| format!("failed to start SWIM node: {}", e))?;

    swim.join(&[leader]).await;

    let metadata = PeerMetadata {
        cpu_load: 0.15,
        mem_avail_mb: 4096,
        task_queue_depth: 0,
    };
    swim.update_metadata(metadata).await;
    swim.start().await;

    let system = ActorSystem::new(1, SupervisionStrategy::OneForOne, 3);

    system
        .spawn(
            move |msg| {
                Box::pin(async move {
                    info!(
                        "Worker received task: {:?} from {}",
                        msg.msg_type, msg.sender
                    );

                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                })
            },
            128,
        )
        .await;

    info!("Worker node running on {}, leader: {}", bind, leader);
    info!("Ready to process tasks (press Ctrl+C to stop)");

    tokio::signal::ctrl_c().await.ok();
    info!("Worker shutting down");
    Ok(())
}

async fn run_standalone(
    bind_addr: &str,
    task_name: &str,
    task_count: u64,
    partitions: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    init_tracer("compute-orchestrator-standalone", None);

    let bind: SocketAddr = bind_addr
        .parse()
        .map_err(|e| format!("invalid bind address: {}", e))?;

    let swim = SwimNode::new(bind, SwimConfig::default())
        .await
        .map_err(|e| format!("failed to start SWIM node: {}", e))?;
    swim.start().await;

    let system = ActorSystem::new(0, SupervisionStrategy::OneForOne, 3);
    let metrics = Arc::new(OrchestratorMetrics::new());

    let macro_task = MacroTask {
        id: 1,
        name: task_name.to_string(),
        payload_type: "computation".to_string(),
        data_range: RangeSpec {
            start: 0,
            end: task_count,
        },
        partition_count: partitions,
    };

    let micro_tasks = split_workload(&macro_task);
    info!(
        "Split macro task '{}' into {} micro tasks ({} partitions)",
        task_name,
        micro_tasks.len(),
        partitions
    );

    let metrics_clone = metrics.clone();
    let task_count = micro_tasks.len();
    for task in micro_tasks {
        let m = metrics_clone.clone();
        let pid = system
            .spawn(
                move |_msg| {
                    let metrics = m.clone();
                    Box::pin(async move {
                        metrics
                            .tasks_dispatched
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        metrics
                            .tasks_completed
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    })
                },
                128,
            )
            .await;

        let msg = ActorMessage::new(ProcessId::new(0, 0), pid, MessageType::TaskDispatch, &task)
            .map_err(|e| format!("failed to serialize task: {}", e))?;

        if let Err(e) = system.send(msg).await {
            tracing::error!("Failed to send task: {}", e);
        }
    }

    info!(
        "Dispatched {} micro tasks. Waiting for completion...",
        task_count
    );

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let snapshot = metrics.snapshot();
    info!(
        "Results: {} dispatched, {} completed, {} failed, {} actors",
        snapshot.tasks_dispatched,
        snapshot.tasks_completed,
        snapshot.tasks_failed,
        snapshot.actors_spawned,
    );

    info!("Standalone run complete");
    Ok(())
}
