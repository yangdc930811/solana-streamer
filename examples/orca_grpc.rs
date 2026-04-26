use sol_common::common::constants::{METEORA_DAMM_V2_PROGRAM_ID, METEORA_DLMM_PROGRAM_ID, ORCA_PROGRAM_ID};
use solana_streamer::streaming::event_parser::{DexEvent, Protocol};
use solana_streamer::streaming::event_parser::common::EventType;
use solana_streamer::streaming::event_parser::common::filter::EventTypeFilter;
use solana_streamer::streaming::grpc::ClientConfig;
use solana_streamer::streaming::yellowstone_grpc::{AccountFilter, TransactionFilter, YellowstoneGrpc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    println!("Orca gRPC (solana-streamer)\n");

    let grpc = YellowstoneGrpc::new_with_config(
        std::env::var("GRPC_ENDPOINT").unwrap_or_else(|_| "https://solana-yellowstone-grpc.publicnode.com:443".to_string()),
        Some("a8ec3a9417d347228509703dc686c935966be4d9cacb5eadfe0fd5834723ba1d".to_string()),
        ClientConfig::default(),
    )?;

    let transaction_filter = TransactionFilter {
        account_include: vec![ORCA_PROGRAM_ID.to_string()],
        account_exclude: vec![],
        account_required: vec![],
    };
    let account_filter = AccountFilter {
        account: vec![],
        owner: vec![ORCA_PROGRAM_ID.to_string()],
        filters: vec![],
    };

    let callback = |event: DexEvent| {
        println!("==============> Event: {:?}", event);
    };

    let event_filter = EventTypeFilter {
        include: vec![EventType::OrcaSwap, EventType::OrcaSwapV2],
    };
    grpc.subscribe_events_immediate(
        vec![Protocol::Orca],
        None,
        vec![transaction_filter],
        vec![account_filter],
        Some(event_filter),
        None,
        callback,
        false,
    )
        .await?;

    println!("Press Ctrl+C to stop...\n");
    tokio::signal::ctrl_c().await?;
    grpc.stop().await;
    Ok(())
}
