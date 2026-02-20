use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use solana_transaction_status::option_serializer::OptionSerializer;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::trade;
use crate::config;
use solana_sdk::hash::Hash;

pub async fn manage_my_position(
    client: RpcClient,
    signature: String,
    active_mints: Arc<Mutex<HashSet<String>>>,
    blockhash: Hash,  // ← add this
) -> anyhow::Result<()> {
    let sig = Signature::from_str(&signature)?;

    let tx = client.get_transaction_with_config(
        &sig,
        RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        },
    ).await?;

    let meta = match tx.transaction.meta {
        Some(m) => m,
        None => return Ok(()),
    };

    let pre_balances = match meta.pre_token_balances {
        OptionSerializer::Some(b) => b,
        _ => vec![],
    };

    let post_balances = match meta.post_token_balances {
        OptionSerializer::Some(b) => b,
        _ => vec![],
    };

    let my_wallet = config::MY_WALLET_ADDRESS;

    for post in &post_balances {
        let owner = match &post.owner {
            OptionSerializer::Some(o) => o.clone(),
            _ => continue,
        };

        if owner != my_wallet {
            continue;
        }

        let pre_amount = pre_balances
            .iter()
            .find(|p| p.account_index == post.account_index)
            .map(|p| p.ui_token_amount.amount.parse::<u64>().unwrap_or(0))
            .unwrap_or(0);

        let post_amount = post.ui_token_amount.amount.parse::<u64>().unwrap_or(0);

        if post_amount <= pre_amount {
            continue;
        }

        let received_amount = post_amount - pre_amount;
        let mint_str = post.mint.clone();

        // Filter out SOL and USDC
        if mint_str == "So11111111111111111111111111111111111111112"
            || mint_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        {
            continue;
        }

        // ── DEDUP GUARD ──
        {
            let mut set = active_mints.lock().await;
            if set.contains(&mint_str) {
                println!("   ⏭️  Skipping {} — sell already in progress", &mint_str[..8]);
                return Ok(());
            }
            set.insert(mint_str.clone());
            println!("   💎 Detected New Position: {}", mint_str);
            println!("   💰 Balance: {} (received: {})", post_amount, received_amount);
        }

        println!("   📉 Selling NOW...");

        // ── RETRY LOOP ──
        let mut result = Err(anyhow::anyhow!("Not attempted"));

        for attempt in 1..=5 {
            result = trade::sell_token(&client, &mint_str, received_amount, blockhash).await;

            match &result {
                Ok(_) => break,
                Err(e) => {
                    let msg = e.to_string();
                    let retryable = msg.contains("0xbc4")
                        || msg.contains("Custom(3012)")
                        || msg.contains("AccountNotInitialized")
                        || msg.contains("0x1787")
                        || msg.contains("Custom(6023)")
                        || msg.contains("NotEnoughTokens");

                    if retryable && attempt < 5 {
                        println!("   ⏳ Retrying in 300ms (attempt {}/5)...", attempt);
                        sleep(Duration::from_millis(300)).await;
                        continue;
                    }
                    break;
                }
            }
        }

        match result {
            Ok(sig) => println!("   ✅ SOLD! https://solscan.io/tx/{}", sig),
            Err(e)  => eprintln!("   ❌ Sell Failed: {}", e),
        }

        {
            let mut set = active_mints.lock().await;
            set.remove(&mint_str);
        }

        return Ok(());
    }

    Ok(())
}