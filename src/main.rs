use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::env;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    version: u32,
    merkleroot: String,
    time: u32,
    nonce: u32,
    bits: String,
    previousblockhash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ScriptPubKey {
    asm: String,
    desc: String,
    hex: String,
    address: Option<String>,
    r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ScriptSig {
    asm: String,
    hex: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vin {
    coinbase: Option<String>,
    txid: Option<String>,
    vout: Option<u32>,
    scriptSig: Option<ScriptSig>,
    sequence: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vout {
    value: f64,
    n: u32,
    scriptPubKey: ScriptPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    txid: String,
    hash: String,
    version: u32,
    size: u32,
    vsize: u32,
    locktime: u32,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
}

fn hash_string(data: String, mut hasher: Sha256) -> String {
    // Update the hasher with the data
    hasher.update(data);

    // Finalize the hasher and retrieve the hash
    let result = hasher.finalize();

    // Convert the hash into a byte array and return it
    format!("{:x}", result)
}

fn hash_binary(data: Vec<u8>, mut hasher: Sha256) -> String {
    // Update the hasher with the data
    hasher.update(data);

    // Finalize the hasher and retrieve the hash
    let result = hasher.finalize();

    // Convert the hash into a byte array and return it
    format!("{:x}", result)
}

fn merkle_proof(root: String, txid_list: Vec<String>) -> bool {

    let mut hashes: Vec<String> = Vec::new();

    for hash in txid_list {
        hashes.push(convert_endianness(&hash.clone()));
    }

    while hashes.len() > 1 {
        if hashes.len() % 2 != 0 {
            hashes.push(hashes.last().unwrap().clone());
        }

        let mut new_hashes = Vec::new();

        for i in 0..hashes.len() / 2 {
            let mut hasher = Sha256::new();

            let mut first = hex::decode(hashes[i * 2].clone()).unwrap();

            let second = hex::decode(hashes[i * 2 + 1].clone()).unwrap();

            first.extend(second);

            let hash = hash_binary(first, hasher.clone());

            let second_hash = hash_binary(hex::decode(hash).unwrap(), hasher);

            new_hashes.push(second_hash);
        }

        hashes = new_hashes;
    }

    let calculated = convert_endianness(&hashes[0]);

    println!("Merkle root: {}",  calculated);
    println!("Root: {}", root);

    return calculated == root;
}

fn find_wtxid_root (transaction: &Transaction) -> Result<String, &'static str> {
    for vout in &transaction.vout {
        if let Some(hex) = vout.scriptPubKey.hex.strip_prefix("6a24aa21a9ed") {
            return Ok(hex.to_string());
        }
    }
    Err("Could not find wtxid root in transaction")
}

fn convert_endianness(hex_string: &str) -> String {
    // Convert hexadecimal string to bytes
    let bytes_data = hex::decode(hex_string).expect("Invalid hexadecimal string");

    // Reverse the byte order for little endian
    let reversed_bytes: Vec<u8> = bytes_data.iter().rev().cloned().collect();

    // Convert reversed bytes back to hexadecimal string
    let reversed_hex_string = hex::encode(reversed_bytes);
    reversed_hex_string
}


fn main() {

    let args: Vec<String> = env::args().collect();

    // Check if there are exactly two arguments
    if args.len() != 3 {
        eprintln!("Usage: {} <header-json> <transaction-json>", args[0]);
        return;
    }

    // Open the JSON file
    let file = File::open(&args[1]).expect("Failed to open file");

    // Create a buffered reader
    let reader = BufReader::new(file);

    // Deserialize the JSON data into your structure
    let block: Block = serde_json::from_reader(reader).expect("Failed to parse JSON");

    let file = File::open(&args[2]).expect("Failed to open file");

    let reader = BufReader::new(file);

    let transactions: Vec<Transaction> = serde_json::from_reader(reader).expect("Failed to parse JSON");

    let wtxid_root = find_wtxid_root(&transactions[0]).unwrap();

    let txids = transactions.iter().map(|x| x.txid.clone()).collect::<Vec<String>>();

    if !merkle_proof(block.merkleroot, txids) {
        println!("Merkle proof for txids failed");
        //panic!();
    }

    let wtxids = transactions.iter().map(|x| x.hash.clone()).collect::<Vec<String>>();

    if !merkle_proof(wtxid_root, wtxids) {
        println!("Merkle proof wtxids failed");
        panic!();
    }

}
