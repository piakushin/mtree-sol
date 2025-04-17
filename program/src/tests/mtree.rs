// Tests for the Merkle Tree Solana Program
#[cfg(test)]
mod tests {
    use solana_program::{account_info::AccountInfo, clock::Epoch, keccak::hash, pubkey::Pubkey};
    use solana_program_test::*;
    use solana_sdk::{
        account::Account,
        signature::{Keypair, Signer},
    };
    use std::mem;

    // Helper function to create an AccountInfo for testing
    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo {
            key,
            is_signer,
            is_writable,
            lamports: lamports as *mut u64,
            data: data as *mut [u8],
            owner,
            executable: false,
            rent_epoch: Epoch::default(),
        }
    }

    #[test]
    fn test_merkle_tree_initialization() {
        // Create a new merkle tree
        let tree = MerkleTree {
            root: [0; 32],
            leaves: Vec::new(),
        };

        // Check that a new tree has the expected properties
        assert_eq!(tree.root, [0; 32]);
        assert_eq!(tree.leaves.len(), 0);
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn test_insert_leaf() {
        // Create a new merkle tree
        let mut tree = MerkleTree {
            root: [0; 32],
            leaves: Vec::new(),
        };

        // Insert a leaf
        let leaf_data = b"Test leaf";
        tree.insert_leaf(leaf_data);

        // Check that the leaf was added
        assert_eq!(tree.leaves.len(), 1);

        // Verify that the root is updated (should equal the first leaf hash in this case)
        let expected_leaf_hash = hash(leaf_data).to_bytes();
        assert_eq!(tree.root, expected_leaf_hash);
    }

    #[test]
    fn test_multiple_leaves() {
        // Create a new merkle tree
        let mut tree = MerkleTree {
            root: [0; 32],
            leaves: Vec::new(),
        };

        // Insert multiple leaves
        let leaf1 = b"Leaf 1";
        let leaf2 = b"Leaf 2";

        tree.insert_leaf(leaf1);
        let root_after_one = tree.root;

        tree.insert_leaf(leaf2);
        let root_after_two = tree.root;

        // Ensure root changes after adding second leaf
        assert_ne!(root_after_one, root_after_two);
        assert_eq!(tree.leaves.len(), 2);

        // Manually calculate what the root should be to verify
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
        let mut data = vec![0; 1000];

        // Create account info
        let merkle_account = create_account_info(
            &merkle_pubkey,
            false,
            true,
            &mut lamports,
            &mut data,
            &program_id,
        );

        // Create accounts array
        let accounts = vec![merkle_account];

        // Create instruction data
        let leaf_data = b"Test instruction";
        let instruction = MerkleInstruction::InsertLeaf {
            data: leaf_data.to_vec(),
        };
        let instruction_data = instruction.try_to_vec().unwrap();

        // Process the instruction
        let result = process_instruction(&program_id, &accounts, &instruction_data);

        // Check the result
        assert!(result.is_ok());

        // Deserialize the account data to check if the leaf was inserted
        let merkle_tree = MerkleTree::try_from_slice(&accounts[0].data.borrow()).unwrap();

        // Check that the leaf was inserted
        assert_eq!(merkle_tree.leaves.len(), 1);

        // Check that the root was updated
        let expected_leaf_hash = hash(leaf_data).to_bytes();
        assert_eq!(merkle_tree.root, expected_leaf_hash);
    }

    #[test]
    fn test_tree_with_odd_number_of_leaves() {
        // Create a new merkle tree
        let mut tree = MerkleTree {
            root: [0; 32],
            leaves: Vec::new(),
        };

        // Insert three leaves
        let leaf1 = b"Leaf 1";
        let leaf2 = b"Leaf 2";
        let leaf3 = b"Leaf 3";

        tree.insert_leaf(leaf1);
        tree.insert_leaf(leaf2);
        tree.insert_leaf(leaf3);

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
