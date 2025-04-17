# Solana Merkle Tree Program

A Solana program that implements a Merkle tree structure and merkle root computation on the Solana blockchain.

## Overview

This project implements a Solana program that maintains a Merkle tree data structure on-chain. It allows users to:

1. Insert leaf nodes into the tree
2. Receive event notifications with the updated root after each insertion

## Key Components

### Program Instructions

The program supports one main instruction:

**InsertLeaf**: Adds a new leaf to the tree and recalculates the root hash


### Event Emission

After each leaf insertion, the program emits an event message containing the updated Merkle root.

## Installation

### Prerequisites

- Rust
- Solana CLI tools

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/piakushin/mtree-sol.git
   cd mtree-sol
   ```

2. Build the program:
   ```bash
   cargo build-bpf
   ```

3. Deploy to a local test validator:
   ```bash
   solana program deploy target/deploy/merkle_tree.so
   ```

4. Run client to insert data:
   ```bash
   cargo run -- "some leaf data"
   ```

## Testing

Run the program tests with:

```bash
cargo test-sbf
```
