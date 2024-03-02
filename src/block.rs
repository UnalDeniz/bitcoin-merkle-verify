use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub struct Block {
    header: Header,
    transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Header {
    version: u32,
    merkleroot: String,
    time: u32,
    nonce: u32,
    bits: String,
    previousblockhash: String,
}

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
    fn find_wtxid_root(&self) -> Result<String, &'static str> {
        for vout in &self.transactions[0].vout {
            if let Some(hex) = vout.script_pub_key.hex.strip_prefix("6a24aa21a9ed") {
                return Ok(hex.to_string());
            }
        }
        Err("Could not find wtxid root in transaction")
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
                let hasher = Sha256::new();

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

        println!("Merkle root: {}", calculated);
        println!("Root: {}", root);

        return calculated == root;
    }

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

        if !Block::merkle_proof(self.header.merkleroot.clone(), txids) {
            println!("Merkle proof for txids failed");
            return false;
        }

        if !Block::merkle_proof(wtxid_root, wtxids) {
            println!("Merkle proof wtxids failed");
            return false;
        }
        true
    }

    pub fn new(header: Header, transactions: Vec<Transaction>) -> Block {
        Block {
            header,
            transactions,
        }
    }
}

fn hash_binary(data: Vec<u8>, mut hasher: Sha256) -> String {
    hasher.update(data);

    let result = hasher.finalize();

    format!("{:x}", result)
}

fn convert_endianness(hex_string: &str) -> String {
    let bytes_data = hex::decode(hex_string).expect("Invalid hexadecimal string");

    let reversed_bytes: Vec<u8> = bytes_data.iter().rev().cloned().collect();

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

#[derive(Serialize, Deserialize, Debug)]
struct Vin {
    coinbase: Option<String>,
    txid: Option<String>,
    vout: Option<u32>,
    #[serde(rename = "scriptSig")]
    script_sig: Option<ScriptSig>,
    sequence: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vout {
    value: f64,
    n: u32,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: ScriptPubKey,
}
