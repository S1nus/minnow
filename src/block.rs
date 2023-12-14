use crate::{transaction::*, state::*, errors::EasyFraudError};
use monotree::Hash;
pub struct IncomingBlock {
    pub transactions: Vec<[u8; 72]>,
}

pub struct TxnISRPair([u8; 72], Hash);
pub struct Header {
    pub apphash: Option<Hash>,
}
pub struct OutgoingBlock {
    pub header: Header,
    pub pairs: Vec<TxnISRPair>,
}

impl IncomingBlock {
    pub fn process(&self, state: &mut State) -> Result<OutgoingBlock, EasyFraudError> {
        let mut outgoing_block = OutgoingBlock{
            header: Header{
                apphash: state.root,
            },
            pairs: vec![],
        };
        self.transactions.iter().try_for_each(|d| -> Result<(), EasyFraudError> {
            let st = SignedTransaction::deserialize(*d)
                .map_err(|e|{
                    return Ok::<(), EasyFraudError>(())
                }).unwrap(); //this should be safe, the above closure always returns Ok
            /*let txn = st.verify_and_deserialize()
                .map_err(|e| {
                    return Ok::<(), EasyFraudError>(())
                }).unwrap();*/
            let isr = state.verify_and_run_transaction(&st)
                .map_err(|e| { return Ok::<(), EasyFraudError>(())})
                .unwrap();
            if let Some(isr) = isr {
                outgoing_block.pairs.push(TxnISRPair(st.transaction_data, isr))
            }

            Ok(())
        })?;
        outgoing_block.header.apphash = state.root;
        Ok(outgoing_block)
    }
}