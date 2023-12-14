mod transaction;
use transaction::Transaction;
mod errors;
use errors::EasyFraudError;
mod state;
use state::State;
mod block;
use block::{*};
mod utils;
use utils::get_init_chain_request;

fn main() {
}


#[cfg(test)]
mod tests {
    use crate::state::AccountBalancePair;

    use super::*;

    #[test]
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

    use monotree::Monotree;
    use rand::rngs::OsRng;
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

    #[test]
    fn test_block() {
        let mut csprng = OsRng;
        
        let state = State {
            initialized: false,
            chain_id: "mychain".into(),
            tree: Monotree::default(),
            root: None,
            height: 0,
            volatile_root: None,
            volatile_diffs: vec![],
        };

        let genesis_whale: SigningKey = SigningKey::generate(&mut csprng);
        let genesis_account = AccountBalancePair {
            pubkey: genesis_whale.verifying_key().to_bytes(),
            balance: 1000000000,
        };
        let init_chain = get_init_chain_request();     
    }
}