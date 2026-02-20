use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use std::{env, str::FromStr};
use bs58;

// ───────── CONSTANTS ─────────

const PUMP_FUN_PROGRAM: &str   = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const GLOBAL_ACCOUNT: &str     = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
const FEE_RECIPIENT: &str      = "62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV";
const EVENT_AUTHORITY: &str    = "Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1";
const FEE_CONFIG: &str         = "8Wf5TiAheLUqBrKXeYg2JtAFFMWtKdG2BSFgqUcPVwTt";
const FEE_PROGRAM: &str        = "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

// ───────── WALLET ─────────

pub fn load_wallet() -> anyhow::Result<Keypair> {
    let key = env::var("PRIVATE_KEY")?;
    let bytes = bs58::decode(key.trim()).into_vec()?;
    Ok(Keypair::from_bytes(&bytes)?)
}

// ───────── PDA HELPER ─────────

fn derive_bonding_curve(program: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"bonding-curve", mint.as_ref()], program).0
}

// ───────── SELL ─────────

pub async fn sell_token(
    rpc: &RpcClient,
    mint_str: &str,
    amount: u64,
    blockhash: Hash, // ← passed in from cache, no RPC call needed
) -> anyhow::Result<String> {
    let wallet        = load_wallet()?;
    let user          = wallet.pubkey();
    let program       = Pubkey::from_str(PUMP_FUN_PROGRAM)?;
    let mint          = Pubkey::from_str(mint_str.trim())?;
    let token_program = Pubkey::from_str(TOKEN_2022_PROGRAM)?;
    let creator_vault = Pubkey::from_str(crate::config::MY_CREATOR_VAULT)?;

    // All derived locally — zero RPC calls
    let bonding_curve = derive_bonding_curve(&program, &mint);
    let curve_ata     = get_associated_token_address_with_program_id(&bonding_curve, &mint, &token_program);
    let user_ata      = get_associated_token_address_with_program_id(&user, &mint, &token_program);

    println!("   🔑 Mint:          {}", mint);
    println!("   🔑 Bonding Curve: {}", bonding_curve);
    println!("   🔑 Curve ATA:     {}", curve_ata);
    println!("   🔑 User ATA:      {}", user_ata);
    println!("   🔑 Creator Vault: {}", creator_vault);
    println!("   📉 Selling {}...", amount);

    // ── BUILD TRANSACTION ─────────────────────────────────────────────────────
    let mut data = vec![51u8, 230, 133, 164, 1, 127, 131, 173]; // sell discriminator
    data.extend_from_slice(&amount.to_le_bytes()); // amount (u64 LE)
    data.extend_from_slice(&0u64.to_le_bytes());   // min_sol_output = 0

    let accounts = vec![
        AccountMeta::new_readonly(Pubkey::from_str(GLOBAL_ACCOUNT)?,  false), // 1
        AccountMeta::new(        Pubkey::from_str(FEE_RECIPIENT)?,    false), // 2
        AccountMeta::new_readonly(mint,                                false), // 3
        AccountMeta::new(        bonding_curve,                        false), // 4
        AccountMeta::new(        curve_ata,                            false), // 5
        AccountMeta::new(        user_ata,                             false), // 6
        AccountMeta::new(        user,                                 true),  // 7
        AccountMeta::new_readonly(system_program::id(),                false), // 8
        AccountMeta::new(        creator_vault,                        false), // 9
        AccountMeta::new_readonly(token_program,                       false), // 10
        AccountMeta::new_readonly(Pubkey::from_str(EVENT_AUTHORITY)?,  false), // 11
        AccountMeta::new_readonly(program,                             false), // 12
        AccountMeta::new_readonly(Pubkey::from_str(FEE_CONFIG)?,       false), // 13
        AccountMeta::new_readonly(Pubkey::from_str(FEE_PROGRAM)?,      false), // 14
    ];

    let instruction = Instruction { program_id: program, accounts, data };

    let priority = ComputeBudgetInstruction::set_compute_unit_price(1_000_000);
    let limit    = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);

    // Use cached blockhash directly — no RPC call
    let tx = Transaction::new_signed_with_payer(
        &[priority, limit, instruction],
        Some(&user),
        &[&wallet],
        blockhash,
    );

    // Skip preflight — fire directly to validator
    let sig = rpc.send_transaction_with_config(
        &tx,
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..Default::default()
        },
    ).await?;

    Ok(sig.to_string())
}