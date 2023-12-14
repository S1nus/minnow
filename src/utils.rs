use tendermint::Block;
use tendermint::time::Time as TmTime;
use tendermint::block::{
    Size as BlockSize,
};
use tendermint::consensus::Params as ConsensusParams;
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

pub fn get_init_chain_request(app_state_bytes: [u8; 72]) -> RequestInitChain {
    RequestInitChain {
        time: TmTime::now(),
        chain_id: "mychain".into(),
        consensus_params: ConsensusParams {
            // we ignore these parameters but unfortunately need to fill them.
            block: BlockSize {
                max_bytes: 0,
                max_gas: 0,
                time_iota_ms: BlockSize::default_time_iota_ms(),
            },
        },
        validators: Default::default(),
        app_state_bytes: app_state_bytes.to_vec().try_into().unwrap(),
        initial_height: 0,
    }
}