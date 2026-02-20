use futures_util::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::commitment_config::CommitmentConfig;
use chrono::Local;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{processor, config};
use solana_sdk::hash::Hash;
use tokio::sync::RwLock;


pub async fn monitor_my_wallet(
    active_mints: Arc<Mutex<HashSet<String>>>,
    cached_blockhash: Arc<RwLock<Hash>>,  // ← add this
) -> anyhow::Result<()> {
    let ws_url = config::get_ws_url();
    println!("🔌 Connecting to WebSocket: {}", ws_url);

    let client = PubsubClient::new(&ws_url).await?;

    let my_wallet = config::MY_WALLET_ADDRESS.to_string();
    let filter = RpcTransactionLogsFilter::Mentions(vec![my_wallet.clone()]);

    let (mut stream, _unsubscribe) = client
        .logs_subscribe(
            filter,
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::confirmed()),
            },
        )
        .await?;

    println!("✅ WATCHING WALLET: {}", my_wallet);
    println!("waiting for activity...");

    while let Some(msg) = stream.next().await {
        let value = msg.value;
        let logs  = value.logs;
        let log_str = logs.join(" ");

        if log_str.contains("InitializeMint")
            || log_str.contains("Transfer")
            || log_str.contains("Instruction: Buy")
        {
            let now = Local::now();
            println!(
                "\n⏰ [{}] 🔔 Activity Detected on Your Wallet",
                now.format("%H:%M:%S")
            );
            println!("   Tx: https://solscan.io/tx/{}", value.signature);

            let signature       = value.signature.clone();
            let active_mints_c  = Arc::clone(&active_mints);
            let blockhash = *cached_blockhash.read().await;

            tokio::spawn(async move {
                let rpc_client = config::get_rpc_client();

                if let Err(e) = processor::manage_my_position(
                    rpc_client,
                    signature,
                    active_mints_c,
                    blockhash,
                )
                .await
                {
                    eprintln!("   ❌ Manager Error: {}", e);
                }
            });
        }
    }

    Ok(())
}