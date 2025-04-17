use anyhow::Result;
use anyhow::anyhow;
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
    // Setup
    let program_id = get_program_id();
    let payer = get_payer();

    let client = get_client();

    let mtree_account = create_mtree_account(program_id, &payer, &client);

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
    transaction.sign(&[&payer], client.get_latest_blockhash().unwrap());

    // Send and confirm the transaction
    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("Transaction confirmed: {signature}");

    // Extract hash from logs
    let tx = client
        .get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                encoding: None,
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: None,
            },
        )
        .unwrap();
    let logs = tx.transaction.meta.unwrap().log_messages.unwrap();
    let msg_with_root = logs.get(3).unwrap();
    let root = &msg_with_root[(msg_with_root.len() - 62)..];
    println!("===================================================");
    println!("New root hash: {root}");
    println!("===================================================");
    Ok(())
}

fn get_leaf_data() -> Result<Vec<u8>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        Err(anyhow!(
            "Expected one argument: the data to insert into the Merkle tree"
        ))
    } else {
        Ok(args[1].as_bytes().to_vec())
    }
}

fn create_mtree_account(program_id: Pubkey, payer: &Keypair, client: &RpcClient) -> Keypair {
    let mtree_account = Keypair::read_from_file("mtree.json").unwrap();

    if client
        .get_account_with_commitment(&mtree_account.pubkey(), CommitmentConfig::confirmed())
        .unwrap()
        .value
        .is_none()
    {
        // Create account for storing the merkle tree
        let rent = client
            .get_minimum_balance_for_rent_exemption(10000)
            .unwrap(); // Estimate space needed
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
        transaction.sign(
            &[payer, &mtree_account],
            client.get_latest_blockhash().unwrap(),
        );

        // Send and confirm the transaction
        println!("Sending transaction");
        let signature = client.send_and_confirm_transaction(&transaction).unwrap();
        println!("Transaction confirmed: {signature}");
    }
    mtree_account
}

fn get_program_id() -> Pubkey {
    let program_id = Keypair::read_from_file("target/deploy/mtree_program-keypair.json")
        .map_err(|e| anyhow!("Failed to read keypair: {e}"))
        .unwrap()
        .pubkey();
    dbg!(&program_id);
    program_id
}

fn get_client() -> RpcClient {
    let rpc_url = String::from("http://127.0.0.1:8899");
    RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed())
}

fn get_payer() -> Keypair {
    Keypair::read_from_file("QNdq2shSxGJcQU2dJdmR8vUwQD9hFtGVa7jBHJrxPUr.json")
        .map_err(|e| anyhow!("Failed to read keypair: {e}"))
        .unwrap()
}
