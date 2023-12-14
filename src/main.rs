mod transaction;
use transaction::Transaction;
mod errors;
use errors::EasyFraudError;
mod state;
use state::State;
mod block;
use block::{*};

fn main() {
}


#[cfg(test)]
mod tests {
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
}