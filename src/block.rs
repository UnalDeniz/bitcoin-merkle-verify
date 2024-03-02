use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A block in the blockchain.
pub struct Block {
    header: Header,
    transactions: Vec<Transaction>,
}

/// The header of a block.
#[derive(Serialize, Deserialize, Debug)]
pub struct Header {
    version: u32,
    merkleroot: String,
    time: u32,
    nonce: u32,
    bits: String,
    previousblockhash: String,
}

/// A transaction in a block.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    txid: String,
    hash: String,
    version: u32,
    size: u32,
    vsize: u32,
    locktime: u32,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
}

impl Block {
    /// Finds the wtxid root from the first transaction.
    fn find_wtxid_root(&self) -> Result<String, &'static str> {
        for vout in &self.transactions[0].vout {
            if let Some(hex) = vout.script_pub_key.hex.strip_prefix("6a24aa21a9ed") {
                return Ok(hex.to_string());
            }
        }
        Err("Could not find wtxid root in transaction")
    }

    /// Checks if the block is valid by verifying the merkle root against all txids and wtxid merkle root against wtxids.
    pub fn verify(&self) -> bool {
        let wtxid_root = self.find_wtxid_root().unwrap();
        let txids = self
            .transactions
            .iter()
            .map(|x| x.txid.clone())
            .collect::<Vec<String>>();
        let wtxids = self
            .transactions
            .iter()
            .map(|x| x.hash.clone())
            .collect::<Vec<String>>();

        if !merkle_proof(self.header.merkleroot.clone(), txids) {
            println!("Merkle proof for txids failed");
            println!("The transactions does not belong to the given header.");
            return false;
        }
        println!("Merkle proof for txids is successful.");

        if !merkle_proof(wtxid_root, wtxids) {
            println!("Merkle proof for wtxids failed");
            println!("The transactions does not belong to the given header.");
            return false;
        }
        println!("Merkle proof for wtxids is successful.");
        println!("The transactions belong to the given header.");
        true
    }

    pub fn new(header: Header, transactions: Vec<Transaction>) -> Block {
        Block {
            header,
            transactions,
        }
    }
}

/// Calculates the merkle root and verifies it against the given root.
fn merkle_proof(root: String, txid_list: Vec<String>) -> bool {
    let mut hashes: Vec<String> = Vec::new();

    // Convert all txids to little-endian.
    for hash in txid_list {
        hashes.push(convert_endianness(&hash.clone()));
    }

    // Continue until there is only one hash left.
    while hashes.len() > 1 {
        // If the number of hashes is odd, duplicate the last hash.
        if hashes.len() % 2 != 0 {
            hashes.push(hashes.last().unwrap().clone());
        }

        let mut new_hashes = Vec::new();

        for i in 0..hashes.len() / 2 {
            let hasher = Sha256::new();

            // Concatenate the two hashes.
            let mut first = hex::decode(hashes[i * 2].clone()).unwrap();

            let second = hex::decode(hashes[i * 2 + 1].clone()).unwrap();

            first.extend(second);

            // Double hash the concatenated hashes.
            let hash = hash_binary(first, hasher.clone());

            let second_hash = hash_binary(hex::decode(hash).unwrap(), hasher);

            new_hashes.push(second_hash);
        }

        hashes = new_hashes;
    }

    // Convert the calculated merkle root to big-endian.
    let calculated = convert_endianness(&hashes[0]);

    println!("The calculated merkle root is: {}", calculated);
    println!("The given merkle root is: {}", root);

    return calculated == root;
}

/// Creates a SHA-256 hash from a binary data.
fn hash_binary(data: Vec<u8>, mut hasher: Sha256) -> String {
    hasher.update(data);

    let result = hasher.finalize();

    format!("{:x}", result)
}

/// Converts the endianness of a hexadecimal string.
fn convert_endianness(hex_string: &str) -> String {
    // Convert the hexadecimal string to bytes.
    let bytes_data = hex::decode(hex_string).expect("Invalid hexadecimal string");

    // Reverse the bytes.
    let reversed_bytes: Vec<u8> = bytes_data.iter().rev().cloned().collect();

    // Convert the reversed bytes to a hexadecimal string.
    let reversed_hex_string = hex::encode(reversed_bytes);
    reversed_hex_string
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

/// The input of a transaction.
#[derive(Serialize, Deserialize, Debug)]
struct Vin {
    coinbase: Option<String>,
    txid: Option<String>,
    vout: Option<u32>,
    #[serde(rename = "scriptSig")]
    script_sig: Option<ScriptSig>,
    sequence: u32,
}

/// The output of a transaction.
#[derive(Serialize, Deserialize, Debug)]
struct Vout {
    value: f64,
    n: u32,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: ScriptPubKey,
}
