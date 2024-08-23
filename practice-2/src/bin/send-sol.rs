use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    transport::Result as TransportResult,
};
use solana_client::rpc_client::RpcClient;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;

fn create_memo_instruction(payer_pubkey: &Pubkey, memo_text: &str) -> Instruction {
    let memo_program_id = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")
        .expect("Invalid Memo program ID");

    let accounts = vec![
        solana_sdk::instruction::AccountMeta::new(*payer_pubkey, true),
    ];

    let data = memo_text.as_bytes().to_vec();

    Instruction {
        program_id: memo_program_id,
        accounts,
        data,
    }
}

fn main() -> TransportResult<()> {
    dotenv().ok();

    let private_key = env::var("SECRET_KEY").expect("Add SECRET_KEY to .env!");
    let as_array: Vec<u8> = serde_json::from_str(&private_key).expect("Invalid SECRET_KEY format");
    let sender = Keypair::from_bytes(&as_array).expect("Invalid secret key");

    let connection = RpcClient::new("https://api.devnet.solana.com".to_string());

    println!("🔑 Our public key is: {}", sender.pubkey());

    let recipient = Pubkey::from_str("AeRm8hg3yUemkFtUGpXCBYk4D8z7fnXkhciUajzTdfSS")
        .expect("Invalid recipient public key");
    println!("💸 Attempting to send 0.01 SOL to {}...", recipient);

    let lamports = (0.01 * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;
    let send_sol_instruction = system_instruction::transfer(&sender.pubkey(), &recipient, lamports);

    let memo_text = "Zdarova Illia";
    let memo_instruction = create_memo_instruction(&sender.pubkey(), memo_text);

    let mut transaction = Transaction::new_with_payer(
        &[send_sol_instruction, memo_instruction],
        Some(&sender.pubkey()),
    );

    let recent_blockhash = connection.get_latest_blockhash()?;
    transaction.sign(&[&sender], recent_blockhash);

    let signature = connection.send_and_confirm_transaction(&transaction)?;
    println!("✅ Transaction confirmed, signature: {}", signature);

    Ok(())
}
