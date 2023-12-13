use monotree::{
    Monotree,
    Hash,
};
use tendermint::{
    AppHash,
    block::Height,
};
use tendermint::v0_38::abci::{
    request::{
        InitChain as RequestInitChain,
        PrepareProposal as RequestPrepareProposal,
        ProcessProposal as RequestProcessProposal,
        FinalizeBlock as RequestFinalizeBlock,
    },
    response::{
        InitChain as ResponseInitChain,
        PrepareProposal as ResponsePrepareProposal,
        ProcessProposal as ResponseProcessProposal,
        FinalizeBlock as ResponseFinalizeBlock,
    },
    Request,
    Response,
};
use crate::errors::EasyFraudError;
use crate::transaction::{
    SignedTransaction,
    Transaction,
};

pub struct AccountBalancePair {
    pub pubkey: [u8; 32],
    pub balance: u64,
}

impl AccountBalancePair {
    fn serialize(&self) -> [u8; 40] {
        let mut buf = [0; 40];
        buf[..32].copy_from_slice(&self.pubkey[..]);
        buf[32..].copy_from_slice(&self.balance.to_le_bytes()[..]);
        buf
    }

    fn deserialize(data: [u8; 40]) -> Result<Self, EasyFraudError> {
        Ok(AccountBalancePair {
            pubkey: data[..32].try_into()
                .map_err(|_| EasyFraudError::GenesisAccountDeserialization)?,
            balance: u64::from_le_bytes(data[32..].try_into()
                .map_err(|_| EasyFraudError::GenesisAccountDeserialization)?),
        })
    }
}

pub struct State {
    pub initialized: bool,
    pub chain_id: String,
    pub tree: Monotree,
    pub root: Option<Hash>,
    pub height: u64,
    // keep track of the pre-image of everything we changed,
    // so we can revert back if needed.
    pub volatile_root: Option<Hash>,
    pub volatile_diffs: Vec<AccountBalancePair>,
}

impl State {
    pub fn call(&mut self, req: Request) {
        
        let rsp = match req {
            // handled messages
            Request::InitChain(init_chain) => self.init_chain(init_chain),
            /*Request::Info(_) => Response::Info(self.info()),
            Request::Query(query) => Response::Query(self.query(query.data)),
            Request::DeliverTx(deliver_tx) => Response::DeliverTx(self.deliver_tx(deliver_tx.tx)),
            Request::Commit => Response::Commit(self.commit()),
            // unhandled messages
            Request::Flush => Response::Flush,
            Request::Echo(_) => Response::Echo(Default::default()),
            Request::BeginBlock(_) => Response::BeginBlock(Default::default()),
            Request::CheckTx(_) => Response::CheckTx(Default::default()),
            Request::EndBlock(_) => Response::EndBlock(Default::default()),
            Request::ListSnapshots => Response::ListSnapshots(Default::default()),
            Request::OfferSnapshot(_) => Response::OfferSnapshot(Default::default()),
            Request::LoadSnapshotChunk(_) => Response::LoadSnapshotChunk(Default::default()),
            Request::ApplySnapshotChunk(_) => Response::ApplySnapshotChunk(Default::default()),

            // Note: https://github.com/tendermint/tendermint/blob/v0.37.x/spec/abci/abci%2B%2B_tmint_expected_behavior.md#adapting-existing-applications-that-use-abci
            Request::PrepareProposal(prepare_prop) => Response::PrepareProposal(PrepareProposal {
                txs: prepare_prop.txs,
            }),
            Request::ProcessProposal(..) => {
                Response::ProcessProposal(response::ProcessProposal::Accept)
            }*/
            _ => {
                panic!("unimplemented");
            }
        };

    }

    // verify the transaction against the current state, then execute it
    // save the old diffs
    pub fn verify_and_run_transaction(&mut self, stx: SignedTransaction) -> Result<(bool, Option<Hash>), EasyFraudError> {
        let txn = match stx.verify_and_deserialize() {
            Ok(txn) => txn,
            Err(_) => {
                return Ok((false, None));
            }
        };

        // transaction must have > 0 satoshi
        if txn.amount <= 0 {
            return Ok((false, None));
        }

        let old_sender_balance_leaf: [u8; 32] = self.tree.get(self.root.as_ref(), &txn.sender_pubkey)
            .map_err(|_| EasyFraudError::TreeGetError)?
            .ok_or(EasyFraudError::SenderNotInitialized)?;
        let mut old_sender_balance_buf = [0; 8];
        old_sender_balance_buf.copy_from_slice(&old_sender_balance_leaf[24..]);
        let old_sender_balance = u64::from_le_bytes(old_sender_balance_buf);

        let old_recipient_balance_leaf: Option<[u8; 32]> = self.tree.get(self.root.as_ref(), &txn.recipient_pubkey)
            .map_err(|_| EasyFraudError::TreeGetError)?;
        let mut old_recipient_balance_buf = [0; 8];
        old_recipient_balance_buf.copy_from_slice(&old_recipient_balance_leaf.unwrap_or([0; 32])[24..]);
        let old_recipient_balance = u64::from_le_bytes(old_recipient_balance_buf);
        
        // validate the transaction
        if old_sender_balance <= txn.amount {
            return Ok((false, None))
        }

        let mut new_sender_balance_leaf = [0; 32];
        new_sender_balance_leaf[24..].copy_from_slice(&(old_sender_balance - txn.amount).to_le_bytes()[..]);

        let mut new_recipient_balance_leaf = [0; 32];
        new_recipient_balance_leaf[24..].copy_from_slice(&(old_recipient_balance + txn.amount).to_le_bytes()[..]);

        let first_root = self.tree.insert(self.root.as_ref(), &txn.sender_pubkey, &new_sender_balance_leaf)
            .map_err(|_| EasyFraudError::TreeInsertionError)?;

        let second_root = self.tree.insert(first_root.as_ref(), &txn.recipient_pubkey, &new_recipient_balance_leaf);
        if let Ok(updated_root) = second_root {
            self.root = updated_root;
            // Transaction execution was success. Now save the old diffs.
            self.volatile_diffs.push(AccountBalancePair {
                pubkey: txn.sender_pubkey,
                balance: old_sender_balance,
            });
            self.volatile_diffs.push(AccountBalancePair {
                pubkey: txn.recipient_pubkey,
                balance: old_recipient_balance,
            });
            return Ok((true, updated_root));
        }

        // second insertion failed. must revert first.
        let reverted_root = self.tree.insert(first_root.as_ref(), &txn.sender_pubkey, &old_sender_balance_leaf)
            .map_err(|_| EasyFraudError::CouldNotRevert)?;
        self.root = reverted_root;
        Ok((false, None))
    }

    pub fn init_chain(&mut self, req: RequestInitChain) -> Result<Response, EasyFraudError> {
        if req.chain_id != self.chain_id {
            return Err(EasyFraudError::ChainIDMismatch)
        }
        if req.initial_height.value() != 1 {
            return Err(EasyFraudError::InvalidGenesisHeight)
        }

        req.app_state_bytes
            .chunks_exact(40)
            .try_for_each(|chunk| {
                let pair = AccountBalancePair::deserialize(chunk.try_into()
                    .map_err(|_| EasyFraudError::GenesisAccountDeserialization)?)?;
                let mut balance_buf = [0; 32];
                balance_buf[24..].copy_from_slice(&pair.balance.to_le_bytes());
                let new_root = self.tree.insert(self.root.as_ref(), &pair.pubkey, &balance_buf)
                    .map_err(|_| EasyFraudError::TreeInsertionError)?;
                self.root = new_root;
                Ok(())
            })?;

        let app_hash = self.root.ok_or(EasyFraudError::NullApphash)?.to_vec();

        Ok(Response::InitChain(ResponseInitChain{
            consensus_params: None,
            validators: vec![],
            app_hash: AppHash::try_from(app_hash)
                .map_err(|_| EasyFraudError::InvalidGenesisAppHash)?,
        }))
    }
}