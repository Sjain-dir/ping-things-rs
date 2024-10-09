use crate::bench::Bench;
use crate::config::PingThingsArgs;
use crate::state_listeners::ChainListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::info;

mod bench;
mod config;
mod state_listeners;
mod tx_sender;

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();
    let config = PingThingsArgs::new();

    info!("starting with config {:?}", config);

    let cancellation_token = CancellationToken::new();

    // wait for end signal
    let cancellation_token_clone = cancellation_token.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => {
                println!("ctrl-c received, shutting down");
                cancellation_token_clone.cancel();
            }
            Err(e) => {
                println!("ctrl-c error: {:?}", e);
            }
        }
    });

    let chain_listener =
        ChainListener::new(config.rpc_for_read.clone(), cancellation_token.clone());
    let bench = Bench::new(config, cancellation_token.clone());
    bench
        .start(
            chain_listener.current_slot.clone(),
            chain_listener.recent_blockhash.clone(),
        )
        .await;
    cancellation_token.cancel();
    let _ = chain_listener.hdl.await;
    info!("exiting main");
}
