use dotenvy::dotenv;
use std::env;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use solana_sdk::hash::Hash;

mod config;
mod listener;
mod processor;
mod trade;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load .env file
    dotenv().ok();

    // 2. Clear terminal screen
    print!("\x1B[2J\x1B[1;1H");

    println!("==========================================");
    println!("   🤖 SOLANA MANAGER BOT v1.0");
    println!("   👀 Mode: WATCHING YOUR WALLET");
    println!("   🔑 Wallet: {}...", &config::MY_WALLET_ADDRESS[0..8]);
    println!("==========================================");

    // 3. Check for API Key
    if env::var("HELIUS_API_KEY").is_err() {
        eprintln!("❌ ERROR: HELIUS_API_KEY not found in .env file");
        return Ok(());
    }

    // 4. Shared blockhash cache — refreshed every 2 seconds in the background.
    //    Eliminates one RPC round trip from every sell transaction.
    let cached_blockhash: Arc<RwLock<Hash>> = Arc::new(RwLock::new(Hash::default()));

    let blockhash_cache = Arc::clone(&cached_blockhash);
    tokio::spawn(async move {
        loop {
            let rpc = config::get_rpc_client();
            match rpc.get_latest_blockhash().await {
                Ok(bh) => {
                    *blockhash_cache.write().await = bh;
                }
                Err(e) => {
                    eprintln!("⚠️  Blockhash refresh failed: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    // 5. Shared dedup guard
    let active_mints: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    // 6. Start the Listener
    if let Err(e) = listener::monitor_my_wallet(active_mints, cached_blockhash).await {
        eprintln!("❌ Listener Crashed: {}", e);
    }

    Ok(())
}