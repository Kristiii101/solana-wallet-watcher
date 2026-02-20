use std::env;
use solana_client::nonblocking::rpc_client::RpcClient;

// --- USER SETTINGS ---
// ⚠️ REPLACE THIS WITH YOUR WALLET'S PUBLIC KEY
// This is the wallet the bot will watch for "Buys" or "Mints".
pub const MY_WALLET_ADDRESS: &str = "9UDd8wQ31ghDw61ryxVvd6DggurEaf2NXdTiWwJKVzby";

// Your creator vault — derived from your wallet, constant across all your coins
pub const MY_CREATOR_VAULT: &str = "9FzHswRcStccasJgNi3Z95VbBLKbpyYfpPMwrS5M1V4c";

// 1. Get API Key from .env
pub fn get_api_key() -> String {
    env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY missing in .env file")
}

// 2. Create RPC Client (For sending transactions)
pub fn get_rpc_client() -> RpcClient {
    let api_key = get_api_key();
    // Using Helius Mainnet RPC
    let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);
    RpcClient::new(rpc_url)
}

// 3. Get WebSocket URL (For listening to events)
pub fn get_ws_url() -> String {
    let api_key = get_api_key();
    // Using Helius Mainnet WSS
    format!("wss://mainnet.helius-rpc.com/?api-key={}", api_key)
}