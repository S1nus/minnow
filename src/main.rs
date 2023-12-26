mod transaction;
use transaction::{Transaction, SignedTransaction};
mod errors;
use errors::EasyFraudError;
mod state;
use state::State;
mod block;
use block::{*};
mod utils;
use utils::*;

use celestia_types::{Commitment};
use celestia_types::nmt::{Namespace};

fn main() {
}


#[cfg(test)]
mod tests {
    use std::fmt;

    use crate::{state::AccountBalancePair, transaction::SignedTransaction};

    use super::*;

    use monotree::Monotree;
    use rand::{rngs::OsRng, Rng};
    use ed25519_dalek::{
        VerifyingKey,
        SigningKey,
        Signature,
        Signer,
        Verifier, ed25519::signature::Keypair,
    };
    use tendermint::v0_38::abci::{
        request::{
            InitChain as RequestInitChain,
        }
    };

    fn test_serialize_transaction() {
        let t = Transaction{
            sender_pubkey: [1; 32],
            recipient_pubkey: [2; 32],
            amount: 3000,
        };
        println!("transaction: {:?}", t);
        let bytes = t.serialize();
        println!("serialized bytes: {:?}", bytes);
        let deserialized = Transaction::deserialize(bytes);
        println!("deserialized txn: {:?}", deserialized);
    }

    // full lifecycle test
    #[test]
    fn test_block() {
        let mut csprng = OsRng;
        
        let mut state = State {
            initialized: false,
            chain_id: "mychain".into(),
            tree: Monotree::default(),
            root: None,
            current_block: None,
            height: 0,
            volatile_root: None,
            volatile_diffs: vec![],
        };

        let genesis_whale: SigningKey = SigningKey::generate(&mut csprng);
        let genesis_account = AccountBalancePair {
            pubkey: genesis_whale.verifying_key().to_bytes(),
            balance: 1000000000,
        };
        let mut init_chain = RequestInitChain::default();
        println!("{:?}", init_chain);
        init_chain.chain_id = "mychain".into();
        init_chain.app_state_bytes = genesis_account.serialize().to_vec().try_into().unwrap();
        println!("root: {:?}", state.root);
        state.init_chain(init_chain).unwrap();
        println!("root: {:?}", state.root);
        let balance = state.tree.get(state.root.as_ref(), &genesis_whale.verifying_key().to_bytes()).unwrap();
        println!("{:?}", balance);
        println!("balance {}", leaf_to_num(&balance.unwrap()));

        let recipients: Vec<SigningKey> = std::iter::repeat_with(|| SigningKey::generate(&mut csprng)).take(100).collect();
        let key_bytes = genesis_whale.verifying_key().to_bytes();
        let block_txns = recipients.iter().map(|r| {
            Transaction {
                sender_pubkey: key_bytes.clone(),
                recipient_pubkey: r.verifying_key().to_bytes(),
                amount: csprng.gen_range(1000..=3000),
            }.sign(&genesis_whale).serialize()
        }).collect::<Vec<[u8; 136]>>();
        let incoming_block = IncomingBlock {
            signed_transactions: block_txns,
        };
        let outgoing_block = incoming_block.process(&mut state).unwrap();
        //println!("Last ISR: {:?}", outgoing_block.pairs.last().unwrap().1);
        //println!("block header: {:?}", outgoing_block.header.apphash.unwrap());
        //println!("outgoing block: {:?}", outgoing_block);
        let blob = pairs_into_blob(outgoing_block.pairs);
        let namespace = Namespace::new(0, b"beemovie").unwrap();
        let commitment = Commitment::from_shares(namespace, &blob).unwrap();
        println!("Commitment: {:?}", commitment);
    }

    #[test]
    fn test_revert() {
        let mut csprng = OsRng;
        
        let mut state = State {
            initialized: false,
            chain_id: "mychain".into(),
            tree: Monotree::default(),
            root: None,
            current_block: None,
            height: 0,
            volatile_root: None,
            volatile_diffs: vec![],
        };

        let genesis_whale: SigningKey = SigningKey::generate(&mut csprng);
        let genesis_account = AccountBalancePair {
            pubkey: genesis_whale.verifying_key().to_bytes(),
            balance: 1000000000,
        };
        let mut init_chain = RequestInitChain::default();
        println!("{:?}", init_chain);
        init_chain.chain_id = "mychain".into();
        init_chain.app_state_bytes = genesis_account.serialize().to_vec().try_into().unwrap();
        println!("root: {:?}", state.root);
        state.init_chain(init_chain).unwrap();
        println!("root: {:?}", state.root);
        let balance = state.tree.get(state.root.as_ref(), &genesis_whale.verifying_key().to_bytes()).unwrap();
        println!("{:?}", balance);
        println!("balance {}", leaf_to_num(&balance.unwrap()));

        let recipients: Vec<SigningKey> = std::iter::repeat_with(|| SigningKey::generate(&mut csprng)).take(100).collect();
        let key_bytes = genesis_whale.verifying_key().to_bytes();
        let tx = Transaction {
            sender_pubkey: key_bytes.clone(),
            recipient_pubkey: recipients[0].verifying_key().to_bytes(),
            amount: 3000,
        }.sign(&genesis_whale);

        let old_state = state.root;
        state.verify_and_run_transaction(&tx).unwrap();
        let new_state = state.root;
        assert_ne!(old_state, new_state);
        state.revert_volatile();
        assert_eq!(old_state, state.root);

    }

    #[test]
    fn test_txns_with_invalid() {
        let mut csprng = OsRng;
        
        let mut state = State {
            initialized: false,
            chain_id: "mychain".into(),
            tree: Monotree::default(),
            root: None,
            current_block: None,
            height: 0,
            volatile_root: None,
            volatile_diffs: vec![],
        };

        let genesis_whale: SigningKey = SigningKey::generate(&mut csprng);
        let genesis_account = AccountBalancePair {
            pubkey: genesis_whale.verifying_key().to_bytes(),
            balance: 1000000000,
        };
        let mut init_chain = RequestInitChain::default();
        println!("{:?}", init_chain);
        init_chain.chain_id = "mychain".into();
        init_chain.app_state_bytes = genesis_account.serialize().to_vec().try_into().unwrap();
        println!("root: {:?}", state.root);
        state.init_chain(init_chain).unwrap();
        println!("root: {:?}", state.root);
        let balance = state.tree.get(state.root.as_ref(), &genesis_whale.verifying_key().to_bytes()).unwrap();
        println!("{:?}", balance);
        println!("balance {}", leaf_to_num(&balance.unwrap()));

        let recipients: Vec<SigningKey> = std::iter::repeat_with(|| SigningKey::generate(&mut csprng)).take(100).collect();
        let key_bytes = genesis_whale.verifying_key().to_bytes();
        let mut block_txns = recipients.iter().map(|r| {
            Transaction {
                sender_pubkey: key_bytes.clone(),
                recipient_pubkey: r.verifying_key().to_bytes(),
                amount: csprng.gen_range(1000..=3000),
            }.sign(&genesis_whale).serialize()
        }).collect::<Vec<[u8; 136]>>();
        // break two transactions by changing a byte in the signature
        block_txns[69][78] = 15;
        block_txns[42][78] = 1;
        let incoming_block = IncomingBlock {
            signed_transactions: block_txns,
        };
        let outgoing_block = incoming_block.process(&mut state).unwrap();
        assert_eq!(outgoing_block.pairs.len(), incoming_block.signed_transactions.len() - 2);
    }
}