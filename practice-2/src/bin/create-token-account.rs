use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_client::rpc_config::RpcSendTransactionConfig;
use spl_associated_token_account::instruction as associated_token_instruction;
use std::env;
use std::error::Error;
use std::str::FromStr;
use dotenv::dotenv;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Загружаем приватный ключ из переменной окружения
    let private_key_str = env::var("SECRET_KEY")?;
    let private_key_bytes: Vec<u8> = serde_json::from_str(&private_key_str)?;
    let sender = Keypair::from_bytes(&private_key_bytes)?;

    // Устанавливаем соединение с devnet
    let connection = RpcClient::new_with_commitment("https://api.devnet.solana.com", CommitmentConfig::confirmed());

    println!("🔑 Our public key is: {}", sender.pubkey());

    // Устанавливаем адреса токенов
    let token_mint_account = Pubkey::from_str("9tfbEYTsRohpDcnkBzw5Q7CuPMhmFVHpMsVLrnoULXRx")?;
    let recipient = Pubkey::from_str("G6JwJwECWNpyHPwZuVFCANQ62rm6Y6DVUWEHWUrgfbmW")?;

    // Получаем или создаем связанный токен-аккаунт
    let associated_token_account_address = spl_associated_token_account::get_associated_token_address(&recipient, &token_mint_account);

    let mut tx_instructions = Vec::new();
    let mut signers = Vec::new();

    // Если аккаунт токенов не существует, создаем его
    match connection.get_account(&associated_token_account_address) {
        Ok(_) => {
            println!("Token Account already exists: {}", associated_token_account_address);
        }
        Err(_) => {
            // Создаем новый аккаунт токенов
            let create_account_ix = associated_token_instruction::create_associated_token_account(
                &sender.pubkey(),       // payer
                &recipient,             // owner
                &token_mint_account,    // mint
                &spl_token::id(),       // token program ID
            );
            tx_instructions.push(create_account_ix);
            signers.push(&sender);
        }
    }

    // Получаем последний блокхеш
    let recent_blockhash = connection.get_latest_blockhash()?;

    // Создаем и подписываем транзакцию
    let transaction = Transaction::new_signed_with_payer(
        &tx_instructions,
        Some(&sender.pubkey()),
        &signers,
        recent_blockhash,
    );

    // Отправляем и подтверждаем транзакцию
    connection.send_and_confirm_transaction_with_spinner_and_config(
        &transaction,
        CommitmentConfig::confirmed(),
        RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(solana_sdk::commitment_config::CommitmentLevel::Confirmed),
            ..RpcSendTransactionConfig::default()
        },
    )?;

    // Выводим ссылку на Explorer
    let link = format!(
        "https://explorer.solana.com/address/{}?cluster=devnet",
        associated_token_account_address
    );
    println!("✅ Created token account: {}", link);

    Ok(())
}
