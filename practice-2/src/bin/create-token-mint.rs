use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
    program_pack::Pack,
};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
};
use spl_token::{
    instruction as token_instruction,
    state::Mint,
};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Убедитесь, что dotenv загружается

    // Загружаем приватный ключ из переменной окружения
    let private_key = env::var("SECRET_KEY").expect("Add SECRET_KEY to .env!");

    // Удаляем квадратные скобки и пробелы
    let private_key_str = private_key
        .trim_start_matches('[')
        .trim_end_matches(']')
        .replace(" ", "");

    // Преобразуем строку в вектор байт
    let secret_key_bytes: Vec<u8> = private_key_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    // Проверяем, что вектор байт не пустой
    if secret_key_bytes.is_empty() {
        panic!("SECRET_KEY contains no valid bytes");
    }

    // Преобразуем байты в Keypair
    let sender = Keypair::from_bytes(&secret_key_bytes)?;

    // Устанавливаем соединение с devnet
    let connection = RpcClient::new("https://api.devnet.solana.com".to_string());

    // Отображаем публичный ключ
    println!("🔑 Our public key is: {}", sender.pubkey());

    // Создаем новый Keypair для аккаунта хранения токенов
    let token_account = Keypair::new();
    let rent_exemption = connection.get_minimum_balance_for_rent_exemption(Mint::LEN)?;

    // Создаем инструкцию для создания аккаунта токенов
    let create_account_ix = system_instruction::create_account(
        &sender.pubkey(),
        &token_account.pubkey(),
        rent_exemption,
        Mint::LEN as u64,
        &spl_token::id(),
    );

    // Инициализируем токен
    let initialize_mint_ix = token_instruction::initialize_mint(
        &spl_token::id(),
        &token_account.pubkey(),
        &sender.pubkey(),  // Владелец токена
        None,
        2,
    )?;

    // Получаем последний блокхеш
    let recent_blockhash = connection.get_latest_blockhash()?;

    // Создаем и подписываем транзакцию
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, initialize_mint_ix],
        Some(&sender.pubkey()),
        &[&sender, &token_account],
        recent_blockhash,
    );

    // Отправляем и подтверждаем транзакцию
    connection.send_and_confirm_transaction_with_spinner_and_config(
        &transaction,
        CommitmentConfig {
            commitment: CommitmentLevel::Processed,
        },
        RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(CommitmentLevel::Processed),
            ..RpcSendTransactionConfig::default()
        },
    )?;

    // Выводим ссылку на Explorer
    let link = format!(
        "https://explorer.solana.com/address/{}?cluster=devnet",
        token_account.pubkey()
    );
    println!("✅ Token Mint: {}", link);

    Ok(())
}
