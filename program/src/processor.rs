use borsh::BorshDeserialize;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    keccak::hash,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::MTreeInstruction, state::MTree};

pub struct Processor;
impl Processor {
    pub(crate) fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = MTreeInstruction::try_from_slice(instruction_data)?;

        match instruction {
            MTreeInstruction::InsertLeaf { data } => {
                Self::process_insert_leaf(program_id, accounts, data)?
            }
        }

        Ok(())
    }

    fn process_insert_leaf(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: Vec<u8>,
    ) -> ProgramResult {
        let signer = next_account_info(&mut accounts.iter())?;
        msg!("Signer: {}", signer.key);

        let mut tree = if let Ok(tree) = MTree::try_from_slice(&signer.data.borrow()) {
            tree
        } else {
            MTree {
                root: [0; 32],
                leaves: Vec::new(),
            }
        };
        msg!("MTree decoded");

        insert_leaf(&mut tree, data.as_slice());
        msg!("Root hash updated: {}", hex::encode(tree.root));
        msg!("Depth: {}", depth(&tree));

        // Serialize the updated state back to the account
        let serialized_data = borsh::to_vec(&tree)?;

        // Ensure the account has enough space
        if serialized_data.len() > signer.data_len() {
            msg!(
                "Space needed: {}, on account: {}",
                serialized_data.len(),
                signer.data_len()
            );
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Save the updated merkle tree back to the account

        let mut data = signer.data.borrow_mut();
        data[..serialized_data.len()].copy_from_slice(&serialized_data);
        msg!("MTree updated");

        Ok(())
    }
}

pub fn depth(tree: &MTree) -> usize {
    let leaf_count = tree.leaves.len();
    if leaf_count == 0 {
        return 0;
    }
    let mut depth = 0;
    let mut nodes = leaf_count;
    while nodes > 1 {
        nodes = nodes.div_ceil(2);
        depth += 1;
    }
    depth
}

fn insert_leaf(tree: &mut MTree, data: &[u8]) {
    let data_hash = hash(data).to_bytes();
    tree.leaves.push(data_hash);

    // Recalculate root

    let mut current_level = tree.leaves.clone();

    while current_level.len() > 1 {
        let mut next_level = Vec::new();

        for i in (0..current_level.len()).step_by(2) {
            // If only one left, move leaf to upper level
            if i + 1 < current_level.len() {
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(&current_level[i]);
                combined.extend_from_slice(&current_level[i + 1]);
                let parent = hash(&combined).to_bytes();
                next_level.push(parent);
            } else {
                next_level.push(current_level[i]);
            }
        }
        current_level = next_level;
    }

    tree.root = current_level[0];
}

// Tests for the Merkle Tree Solana Program
#[cfg(test)]
mod tests {
    use crate::entrypoint::process_instruction;

    use super::*;
    use solana_program::{account_info::AccountInfo, clock::Epoch, keccak::hash, pubkey::Pubkey};
    use solana_sdk::signature::{Keypair, Signer};
    use std::{cell::RefCell, rc::Rc};

    // Helper function to create an AccountInfo for testing
    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: Rc<RefCell<&'a mut u64>>,
        data: Rc<RefCell<&'a mut [u8]>>,
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo {
            key,
            is_signer,
            is_writable,
            lamports,
            data,
            owner,
            executable: false,
            rent_epoch: Epoch::default(),
        }
    }

    #[test]
    fn test_insert_leaf() {
        // Create a new merkle tree
        let mut tree = MTree {
            root: [0; 32],
            leaves: Vec::new(),
        };

        // Insert a leaf
        let leaf_data = b"Test leaf";
        insert_leaf(&mut tree, leaf_data.as_slice());

        // Check that the leaf was added
        assert_eq!(tree.leaves.len(), 1);

        // Verify that the root is updated
        let expected_leaf_hash = hash(leaf_data).to_bytes();
        assert_eq!(tree.root, expected_leaf_hash);
    }

    #[test]
    fn test_multiple_leaves() {
        // Create a new merkle tree
        let mut tree = MTree {
            root: [0; 32],
            leaves: Vec::new(),
        };

        // Insert multiple leaves
        let leaf1 = b"Leaf 1";
        let leaf2 = b"Leaf 2";

        insert_leaf(&mut tree, leaf1.as_slice());
        let root_after_one = tree.root;

        insert_leaf(&mut tree, leaf2.as_slice());
        let root_after_two = tree.root;

        // Ensure root changes after adding second leaf
        assert_ne!(root_after_one, root_after_two);
        assert_eq!(tree.leaves.len(), 2);

        let hash1 = hash(leaf1).to_bytes();
        let hash2 = hash(leaf2).to_bytes();

        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(&hash1);
        combined.extend_from_slice(&hash2);
        let expected_root = hash(&combined).to_bytes();

        assert_eq!(tree.root, expected_root);
    }

    #[test]
    fn test_process_instruction() {
        // Create program ID
        let program_id = Pubkey::new_unique();

        // Create merkle account
        let merkle_account_keypair = Keypair::new();
        let merkle_pubkey = merkle_account_keypair.pubkey();

        // Set up account data
        let mut lamports = 100000;
        let mut data = vec![0; 10000];

        // Create account info
        let merkle_account = create_account_info(
            &merkle_pubkey,
            false,
            true,
            Rc::new(RefCell::new(&mut lamports)),
            Rc::new(RefCell::new(&mut data)),
            &program_id,
        );

        // Create accounts array
        let accounts = vec![merkle_account];

        // Create instruction data
        let leaf_data = b"Test instruction";
        let instruction = MTreeInstruction::InsertLeaf {
            data: leaf_data.to_vec(),
        };
        let instruction_data = borsh::to_vec(&instruction).unwrap();

        // Process the instruction
        let result = process_instruction(&program_id, &accounts, &instruction_data);

        // Check the result
        assert!(result.is_ok());

        // // Deserialize the account data to check if the leaf was inserted
        // dbg!(accounts[0].data.borrow().len());
        // let merkle_tree = MTree::try_from_slice(&accounts[0].data.borrow()).unwrap();

        // // Check that the leaf was inserted
        // assert_eq!(merkle_tree.leaves.len(), 1);

        // // Check that the root was updated
        // let expected_leaf_hash = hash(leaf_data).to_bytes();
        // assert_eq!(merkle_tree.root, expected_leaf_hash);
    }

    #[test]
    fn test_tree_with_odd_number_of_leaves() {
        // Create a new merkle tree
        let mut tree = MTree {
            root: [0; 32],
            leaves: Vec::new(),
        };

        // Insert three leaves
        let leaf1 = b"Leaf 1";
        let leaf2 = b"Leaf 2";
        let leaf3 = b"Leaf 3";

        insert_leaf(&mut tree, leaf1.as_slice());
        insert_leaf(&mut tree, leaf2.as_slice());
        insert_leaf(&mut tree, leaf3.as_slice());

        // Check that all leaves were added
        assert_eq!(tree.leaves.len(), 3);

        // Manually calculate what the root should be with 3 leaves
        let hash1 = hash(leaf1).to_bytes();
        let hash2 = hash(leaf2).to_bytes();
        let hash3 = hash(leaf3).to_bytes();

        // First combine hash1 and hash2
        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(&hash1);
        combined.extend_from_slice(&hash2);
        let parent1 = hash(&combined).to_bytes();

        // Then combine parent1 with hash3
        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(&parent1);
        combined.extend_from_slice(&hash3);
        let expected_root = hash(&combined).to_bytes();

        // Verify the result
        assert_eq!(tree.root, expected_root);
    }
}
