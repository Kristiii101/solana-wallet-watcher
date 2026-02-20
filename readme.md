# Solana Wallet Watcher Bot

A high-speed Solana bot that monitors a wallet for token purchases and automatically executes sell transactions on Pump.fun using Token-2022.

## What It Does

- Watches a Solana wallet in real time via WebSocket
- Detects incoming token positions by parsing transaction metadata
- Automatically sells tokens on Pump.fun the moment a buy is detected
- Skips preflight simulation for maximum speed
- Caches the latest blockhash in the background to eliminate RPC round trips on every sell
- Retries automatically on transient RPC failures

## How It Works

1. **Listener** connects to Helius WebSocket and subscribes to logs mentioning your wallet
2. **Processor** fetches the confirmed transaction, parses token balance changes, and identifies new positions owned by your wallet
3. **Trade** constructs and sends the Pump.fun sell instruction with all accounts derived locally — no extra RPC calls needed

## Architecture

```
main.rs         — entry point, starts blockhash cache + listener
listener.rs     — WebSocket log subscription, spawns processor tasks
processor.rs    — parses tx metadata, detects new positions, retry loop
trade.rs        — builds and sends Pump.fun sell instruction
config.rs       — wallet address, RPC/WSS URLs, creator vault
```

## Prerequisites

- Rust (latest stable)
- A [Helius](https://helius.dev) API key (free tier works for testing)
- A Solana wallet with SOL for transaction fees

## Setup

1. Clone the repo
```bash
git clone https://github.com/yourusername/solana-wallet-watcher
cd solana-wallet-watcher
```

2. Create a `.env` file in the root directory
```env
HELIUS_API_KEY=your_helius_api_key_here
PRIVATE_KEY=your_base58_private_key_here
```

3. Update `config.rs` with your wallet details
```rust
pub const MY_WALLET_ADDRESS: &str = "YOUR_WALLET_PUBLIC_KEY";
pub const MY_CREATOR_VAULT: &str  = "YOUR_CREATOR_VAULT_ADDRESS";
```

> To find your creator vault address, make one Pump.fun transaction and check account #9 in the transaction on Solscan.

4. Build and run
```bash
cargo run --release
```

## Configuration

| Constant | Location | Description |
|----------|----------|-------------|
| `MY_WALLET_ADDRESS` | `config.rs` | Wallet to watch for incoming tokens |
| `MY_CREATOR_VAULT` | `config.rs` | Your Pump.fun creator vault PDA |
| `HELIUS_API_KEY` | `.env` | Helius RPC/WSS API key |
| `PRIVATE_KEY` | `.env` | Base58 encoded private key for signing sells |

## Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
anyhow = "1"
dotenvy = "0.15"
chrono = "0.4"
futures-util = "0.3"
bs58 = "0.5"
solana-client = "1.18"
solana-sdk = "1.18"
solana-transaction-status = "1.18"
spl-associated-token-account = "3"
```

## Security

- Never commit your `.env` file
- Never share your `PRIVATE_KEY`
- The `.gitignore` should include `.env`

## Disclaimer

This software is for educational purposes. Use at your own risk. The authors are not responsible for any financial losses incurred through the use of this bot.