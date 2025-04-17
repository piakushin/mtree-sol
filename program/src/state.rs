use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MTree {
    pub root: [u8; 32],
    pub leaves: Vec<[u8; 32]>,
}
