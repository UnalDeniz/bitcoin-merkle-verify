#![warn(missing_docs)]

//! A simple blockchain verification in Rust.

use crate::block::{Block, Header, Transaction};
use std::env;
use std::fs::File;
use std::io::BufReader;

mod block;

/// Reads a header and a list of transactions from JSON files and verifies the block.
fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if the number of arguments is correct.
    if args.len() != 3 {
        eprintln!("Usage: {} <header-json> <transaction-json>", args[0]);
        return;
    }

    let file = File::open(&args[1]).expect("Failed to open file");

    // Create a new buffered reader to read the file.
    let reader = BufReader::new(file);

    // Deserialize the JSON file into a Header struct.
    let header: Header = serde_json::from_reader(reader).expect("Failed to parse JSON");

    let file = File::open(&args[2]).expect("Failed to open file");

    let reader = BufReader::new(file);

    // Deserialize the JSON file into a Transaction struct.
    let transactions: Vec<Transaction> =
        serde_json::from_reader(reader).expect("Failed to parse JSON");

    // Create a new block with the header and transactions.
    let block = Block::new(header, transactions);

    // Verify the block.
    block.verify();
}
