use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
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

fn hash_data(data: String, mut hasher: Sha256) -> String {
    // Update the hasher with the data
    hasher.update(data);

    // Finalize the hasher and retrieve the hash
    let result = hasher.finalize();

    // Convert the hash into a byte array and return it
    format!("{:x}", result)
}

fn merkle_proof(root: String, txid_list: Vec<String>) -> bool {
    let mut hashes = txid_list.clone();

    while hashes.len() > 1 {
        if hashes.len() % 2 != 0 {
            hashes.push(hashes.last().unwrap().clone());
        }

        let mut new_hashes = Vec::new();

        for i in 0..hashes.len() / 2 {
            let mut hasher = Sha256::new();

            let data = format!("{}{}", hashes[i * 2], hashes[i * 2 + 1]);

            let hash = hash_data(data, hasher);

            new_hashes.push(hash);
        }

        hashes = new_hashes;
    }

    hashes[0] == root
}

fn find_wtxid_root (transaction: &Transaction) -> Result<String, &'static str> {
    for vout in &transaction.vout {
        if let Some(hex) = vout.scriptPubKey.hex.strip_prefix("6a24aa21a9ed") {
            return Ok(hex.to_string());
        }
    }
    Err("Could not find wtxid root in transaction")
}

fn main() {
    // Open the JSON file
    let file = File::open("../header.json").expect("Failed to open file");

    // Create a buffered reader
    let reader = BufReader::new(file);

    // Deserialize the JSON data into your structure
    let block: Block = serde_json::from_reader(reader).expect("Failed to parse JSON");

    let file = File::open("../transactions.json").expect("Failed to open file");

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
