use crate::block::{Block, Header, Transaction};
use std::env;
use std::fs::File;
use std::io::BufReader;

mod block;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <header-json> <transaction-json>", args[0]);
        return;
    }

    let file = File::open(&args[1]).expect("Failed to open file");

    let reader = BufReader::new(file);

    let header: Header = serde_json::from_reader(reader).expect("Failed to parse JSON");

    let file = File::open(&args[2]).expect("Failed to open file");

    let reader = BufReader::new(file);

    let transactions: Vec<Transaction> =
        serde_json::from_reader(reader).expect("Failed to parse JSON");

    let block = Block::new(header, transactions);

    block.verify();
}
