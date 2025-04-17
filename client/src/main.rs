use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_instruction;
use solana_sdk::{
    signature::{Keypair, Signer},
    signer::EncodableKey,
    transaction::Transaction,
};

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum MTreeInstruction {
    InsertLeaf { data: Vec<u8> },
}

fn main() -> Result<()> {
    dotenv::dotenv()?;

    // Setup
    let program_id = get_program_id()?;
    let payer = get_payer()?;

    let client = get_client();

    let mtree_account = create_mtree_account(program_id, &payer, &client)?;

    // Build transaction
    let leaf_data = get_leaf_data()?;
    let instruction_data = MTreeInstruction::InsertLeaf { data: leaf_data };

    let insert_leaf_ix = Instruction::new_with_borsh(
        program_id,
        &instruction_data,
        vec![AccountMeta {
            pubkey: mtree_account.pubkey(),
            is_signer: false,
            is_writable: true,
        }],
    );

    let mut transaction = Transaction::new_with_payer(&[insert_leaf_ix], Some(&payer.pubkey()));
    let recent_blockhash = client
        .get_latest_blockhash()
        .with_context(|| "Failed to get latest blockhash")?;
    transaction.sign(&[&payer], recent_blockhash);

    // Send and confirm the transaction
    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("Transaction confirmed: {signature}");

    // Extract hash from logs
    let tx = client.get_transaction_with_config(
        &signature,
        RpcTransactionConfig {
            encoding: None,
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: None,
        },
    )?;
    let logs = tx
        .transaction
        .meta
        .ok_or(anyhow!("Meta is not set"))?
        .log_messages
        .ok_or(anyhow!("Log messages are no set"))?;

    let msg_with_root = logs
        .get(3)
        .ok_or(anyhow!("Log messages are shorter than expected"))?;

    let root = &msg_with_root[(msg_with_root.len() - 62)..];
    println!("===================================================");
    println!("New root hash: {root}");
    println!("===================================================");
    Ok(())
}

fn get_leaf_data() -> Result<Vec<u8>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        bail!("Expected one argument: the data to insert into the Merkle tree")
    } else {
        Ok(args[1].as_bytes().to_vec())
    }
}

fn create_mtree_account(
    program_id: Pubkey,
    payer: &Keypair,
    client: &RpcClient,
) -> Result<Keypair> {
    let mtree_account = Keypair::read_from_file(
        dotenv::var("MTREE_ACCOUNT_KEYPAIR_JSON").expect("Missing keypair for mtree account"),
    )
    .expect("Generate json with keypair for storing MTree and set MTREE_ACCOUNT_KEYPAIR_JSON var");

    if client
        .get_account_with_commitment(&mtree_account.pubkey(), CommitmentConfig::confirmed())
        .unwrap()
        .value
        .is_none()
    {
        // Create account for storing the merkle tree
        let rent = client.get_minimum_balance_for_rent_exemption(10000)?; // Estimate space needed
        let create_account_ix = system_instruction::create_account(
            &payer.pubkey(),
            &mtree_account.pubkey(),
            rent,
            10000,
            &program_id,
        );
        // Add the instruction to new transaction
        let mut transaction =
            Transaction::new_with_payer(&[create_account_ix], Some(&payer.pubkey()));
        transaction.sign(&[payer, &mtree_account], client.get_latest_blockhash()?);

        // Send and confirm the transaction
        println!("Sending transaction");
        let signature = client.send_and_confirm_transaction(&transaction)?;
        println!("Transaction confirmed: {signature}");
    }
    Ok(mtree_account)
}

fn get_program_id() -> Result<Pubkey> {
    let program_id = Keypair::read_from_file(
        dotenv::var("PROGRAM_KEYPAIR_JSON").expect("Missing program keypair file"),
    )
    .map_err(|e| anyhow!("Failed to read keypair: {e}"))?
    .pubkey();
    dbg!(&program_id);
    Ok(program_id)
}

fn get_client() -> RpcClient {
    let rpc_url = dotenv::var("RPC_URL").expect("Missing solana rpc url");
    RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed())
}

fn get_payer() -> Result<Keypair> {
    let payer = Keypair::read_from_file(
        dotenv::var("PAYER_KEYPAIR_JSON").expect("Missing payer keypair file"),
    )
    .map_err(|e| anyhow!("Failed to read keypair: {e}"))?;
    Ok(payer)
}
