use crate::{transaction::*, state::*, errors::EasyFraudError};
use monotree::Hash;
use celestia_types::{Share};
pub struct IncomingBlock {
    pub signed_transactions: Vec<[u8; 136]>,
}

#[derive(Debug)]
pub struct SignedTxnISRPair(pub [u8; 136], pub Hash);
impl SignedTxnISRPair {
    pub fn serialize(&self) -> [u8; 168] {
        let mut buf = [0; 168];
        buf[..136].copy_from_slice(&self.0[..]);
        buf[136..].copy_from_slice(&self.1[..]);
        buf
    }

    pub fn deserialize(data: [u8; 168]) -> Result<Self, EasyFraudError> {
        Ok(SignedTxnISRPair {
            0: data[..136].try_into()
                .map_err(|_| EasyFraudError::SerializePairsError)?,
            1: data[136..].try_into()
                .map_err(|_| EasyFraudError::SerializePairsError)?,
        })
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, EasyFraudError> {
        Ok(SignedTxnISRPair {
            0: data[..136].try_into()
                .map_err(|_| EasyFraudError::SerializePairsError)?,
            1: data[136..].try_into()
                .map_err(|_| EasyFraudError::SerializePairsError)?,
        })
    }
}

#[derive(Debug)]
pub struct Header {
    pub apphash: Option<Hash>,
}
#[derive(Debug)]
pub struct OutgoingBlock {
    pub header: Header,
    pub pairs: Vec<SignedTxnISRPair>,
}

impl IncomingBlock {
    pub fn process(&self, state: &mut State) -> Result<OutgoingBlock, EasyFraudError> {
        let mut outgoing_block = OutgoingBlock{
            header: Header{
                apphash: state.root,
            },
            pairs: vec![],
        };

        // Deserialize, verify_and_run each signed_transaction, filter out the invalid ones, and add the valid ones to the outgoing block
        self.signed_transactions.iter().for_each(|d| {
            //let st = SignedTransaction::deserialize(*d).expect("couldn't deserialize");
            // let txn = deserialize, skip if invalid:
            let st = SignedTransaction::deserialize(*d);
            if let Ok(stx) = st {
                if let Ok(isr) = state.verify_and_run_transaction(&stx) {
                    if let Some(isr) = isr {
                        outgoing_block.pairs.push(SignedTxnISRPair(*d, isr))
                    }
                }

            }
        });
        outgoing_block.header.apphash = state.root;
        Ok(outgoing_block)
    }
}

pub fn pairs_into_blob(pairs: Vec<SignedTxnISRPair>) -> Vec<Share> {
    let mut result = vec![];
    // take 3 SignedTxnISRPairs at a time, pack them into a [u8; 512]
    for i in (0..pairs.len()).step_by(3) {
        let mut buf = [0; 512];
        buf[..168].copy_from_slice(&pairs[i].serialize()[..]);
        if i + 1 >= pairs.len() {
            break;
        }
        buf[168..336].copy_from_slice(&pairs[i+1].serialize()[..]);
        if i + 2 >= pairs.len() {
            break;
        }
        buf[336..504].copy_from_slice(&pairs[i+2].serialize()[..]);
        result.push(Share {
            data: buf
        });
    }
    result
}

// still figuring out how i wanna do this...
/*impl OutgoingBlock {
    pub fn process(&self, state: &mut State) -> Result<(), EasyFraudError> {
        self.pairs.iter().for_each(|d| {
            //let st = SignedTransaction::deserialize(*d).expect("couldn't deserialize");
            // let txn = deserialize, skip if invalid:
            let st = SignedTransaction::deserialize(d.0);
            if let Ok(stx) = st {
                if let Ok(isr) = state.verify_and_run_transaction(&stx) {
                    if let Some(isr) = isr {
                        state.volatile_diffs.push((stx.transaction_data, isr))
                    }
                }

            }
        });
        Ok(())
    }
}*/