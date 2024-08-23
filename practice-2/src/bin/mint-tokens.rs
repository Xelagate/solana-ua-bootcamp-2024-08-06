use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::instruction::mint_to;
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

    // Наш токен имеет 2 десятичных знака
    let minor_units_per_major_units = 10u64.pow(2);

    let token_mint_account = Pubkey::from_str("8CQLxw8KV1m2WB8gqCsc1ME14LQpStnf1cpLPjQQjPwK")?;
    let recipient_associated_token_account = Pubkey::from_str("4oU5kYk4W2fThZJEnamPeS4LxSf1d2uNLigycUnx94Ks")?;

    // Создаем инструкцию на чеканку токенов
    let mint_to_ix = mint_to(
        &spl_token::id(),
        &token_mint_account,
        &recipient_associated_token_account,
        &sender.pubkey(),
        &[],
        10 * minor_units_per_major_units,
    )?;

    // Получаем последний блокхеш
    let recent_blockhash = connection.get_latest_blockhash()?;

    // Создаем и подписываем транзакцию
    let transaction = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    // Отправляем транзакцию
    let signature = connection.send_and_confirm_transaction(&transaction)?;

    // Выводим ссылку на Explorer
    let link = format!(
        "https://explorer.solana.com/tx/{}?cluster=devnet",
        signature
    );
    println!("✅ Success! Mint Token Transaction: {}", link);

    Ok(())
}
