use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum MTreeInstruction {
    InsertLeaf { data: Vec<u8> },
}
