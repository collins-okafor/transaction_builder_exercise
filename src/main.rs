use bitcoin::network::constants::Network;
use bitcoin::util::address::Address;
use bitcoin::consensus::encode::{serialize, deserialize};
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin::blockdata::opcodes::all::{OP_SHA256, OP_EQUAL};
use std::str::FromStr;

fn generate_redeem_script(preimage: &str, transaction: &Transaction) -> String {
    let preimage_bytes = hex::decode(preimage).unwrap();
    let lock_hex = bitcoin::util::bip143::SigHashCache::new(transaction)
        .output_single(0, &Builder::new().push_slice(&preimage_bytes).into_script())
        .script_code(&Builder::new().push_opcode(OP_SHA256).push_slice(&preimage_bytes).push_opcode(OP_EQUAL).into_script())
        .build()
        .to_hex();
    format!("OP_SHA256 {} OP_EQUAL", lock_hex)
}


fn derive_address(redeem_script: &str) -> Address {
    let redeem_script_bytes = hex::decode(redeem_script).unwrap();
    let script = Builder::new().push_slice(&redeem_script_bytes).into_script();
    Address::from_script(&script, Network::Testnet).unwrap()
}

fn construct_transaction(target_address: &Address, amount: u64) -> Transaction {
    let txout = TxOut {
        value: amount,
        script_pubkey: target_address.script_pubkey(),
    };
    Transaction {
        version: 1,
        lock_time: 0,
        input: Vec::new(),
        output: vec![txout],
    }
}

fn construct_spending_transaction(
    prev_transaction: &Transaction,
    redeem_script: &str,
    amount_to_spend: u64,
    change_address: &Address,
) -> Transaction {
    let txid = prev_transaction.txid();
    let txin = TxIn {
        previous_output: txid.into(),
        script_sig: Builder::new().push_slice(&hex::decode(redeem_script).unwrap()).into_script(),
        sequence: 0xFFFFFFFF,
        witness: Vec::new(),
    };
    let txout1 = TxOut {
        value: amount_to_spend,
        script_pubkey: change_address.script_pubkey(),
    };
    let txout2 = TxOut {
        value: prev_transaction.output[0].value - amount_to_spend,
        script_pubkey: prev_transaction.output[0].script_pubkey.clone(),
    };
    Transaction {
        version: 1,
        lock_time: 0,
        input: vec![txin],
        output: vec![txout1, txout2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_redeem_script() {
        let preimage = "427472757374204275696c64657273";
        let redeem_script = generate_redeem_script(preimage);
        assert_eq!(redeem_script, "OP_SHA256 0100000000000000000000000000000000000000000000000000000000000000 OP_EQUAL");
    }

    #[test]
    fn test_derive_address() {
        let redeem_script = "OP_SHA256 0100000000000000000000000000000000000000000000000000000000000000 OP_EQUAL";
        let address = derive_address(redeem_script);
        assert_eq!(address.to_string(), "tb1qzxzjgakmhrqhq0s37lkxrn6j74vqvp3v7r6x2k");
    }

    #[test]
    fn test_construct_transaction() {
        let target_address = Address::from_str("tb1qzxzjgakmhrqhq0s37lkxrn6j74vqvp3v7r6x2k").unwrap();
        let amount = 50000;
        let transaction = construct_transaction(&target_address, amount);
        assert_eq!(transaction.output.len(), 1);
        assert_eq!(transaction.output[0].value, amount);
        assert_eq!(transaction.output[0].script_pubkey, target_address.script_pubkey());
    }

    #[test]
    fn test_construct_spending_transaction() {
        let redeem_script = "OP_SHA256 0100000000000000000000000000000000000000000000000000000000000000 OP_EQUAL";
        let prev_transaction_hex = "010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff04011b0b64197676a9143a609ee60f8bb8be750af949137eaa3aeebd2ec88ac0000000000000000143079a50698a02f2c61a1ed5a58b8a5d2b642ae173f00000000";
        let prev_transaction_bytes = hex::decode(prev_transaction_hex).unwrap();
        let prev_transaction: Transaction = deserialize(&prev_transaction_bytes).unwrap();
        let amount_to_spend = 5000;
        let change_address = derive_address(redeem_script);
        let spending_transaction = construct_spending_transaction(&prev_transaction, redeem_script, amount_to_spend, &change_address);
        assert_eq!(spending_transaction.input.len(), 1);
        assert_eq!(spending_transaction.output.len(), 2);
        assert_eq!(spending_transaction.output[0].value, amount_to_spend);
        assert_eq!(spending_transaction.output[1].value, prev_transaction.output[0].value - amount_to_spend);
        assert_eq!(spending_transaction.output[0].script_pubkey, change_address.script_pubkey());
        assert_eq!(spending_transaction.output[1].script_pubkey, prev_transaction.output[0].script_pubkey);
    }
}

fn main() {
    let preimage = "427472757374204275696c64657273";
    let transaction = Transaction::default(); // Create a default transaction
    let redeem_script = generate_redeem_script(preimage, &transaction);
    let target_address = derive_address(&redeem_script);

    println!("Redeem Script: {}", redeem_script);
    println!("Derived Address: {}", target_address);

    let amount = 50000;
    let transaction = construct_transaction(&target_address, amount);

    println!("Constructed Transaction:\n{}", serialize_hex(&transaction));

    let redeem_script_hex = "OP_SHA256 010000000000000000000000000000000000000000000000000000000000000000 OP_EQUAL";
    let change_address = derive_address(&redeem_script_hex);
    let spending_transaction = construct_spending_transaction(
        &transaction,
        &redeem_script_hex,
        10000,
        &change_address,
    );

    println!("Spending Transaction:\n{}", serialize_hex(&spending_transaction));
}

