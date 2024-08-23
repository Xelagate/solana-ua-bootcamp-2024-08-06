use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use mpl_token_metadata::instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs};
use mpl_token_metadata::types::DataV2;
use std::env;
use std::error::Error;
use std::str::FromStr;
use dotenv::dotenv;

async fn create_metadata() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let private_key_str = env::var("SECRET_KEY")?;
    let private_key_bytes: Vec<u8> = serde_json::from_str(&private_key_str)?;
    let user = Keypair::from_bytes(&private_key_bytes)?;

    let connection = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    let token_metadata_program_id = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")?;
    let token_mint_account = Pubkey::from_str("Hv7pYeh1wkNBhz6E9fqshLg3ZgNuzWtK7aR3CC9eAafX")?;

    let (metadata_pda, _metadata_bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            token_metadata_program_id.as_ref(),
            token_mint_account.as_ref(),
        ],
        &token_metadata_program_id,
    );

    let data = DataV2 {
        name: "My Token".to_string(),
        symbol: "MTK".to_string(),
        uri: "https://example.com/metadata.json".to_string(),
        seller_fee_basis_points: 500,
        creators: None,
        collection: None,
        uses: None,
    };

    let args = CreateMetadataAccountV3InstructionArgs {
        data,
        is_mutable: true,
        collection_details: None,
    };

    let create_metadata_account = CreateMetadataAccountV3 {
        metadata: metadata_pda,
        mint: token_mint_account,
        mint_authority: user.pubkey(),
        payer: user.pubkey(),
        update_authority: (user.pubkey(), true),
        system_program: solana_sdk::system_program::ID,
        rent: None, // Или укажите Pubkey для rent
    };

    let ix = create_metadata_account.instruction(args);

    let recent_blockhash = connection.get_latest_blockhash().await?;
    let mut transaction = Transaction::new_with_payer(&[ix], Some(&user.pubkey()));
    transaction.sign(&[&user], recent_blockhash);

    connection.send_and_confirm_transaction(&transaction).await?;

    let token_mint_link = format!(
        "https://explorer.solana.com/address/{}?cluster=devnet",
        token_mint_account
    );
    println!("✅ Look at the token mint again: {}", token_mint_link);

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = create_metadata().await {
        eprintln!("Error: {}", err);
    }
}
