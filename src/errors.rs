use thiserror::{Error};

#[derive(Error, Debug)]
pub enum EasyFraudError {
    #[error("Transaction deserialization error")]
    TransactionDeserializationError,
    #[error("Error signing")]
    SigningError,
    #[error("ChainID mismatch")]
    ChainIDMismatch,
    #[error("Genesis height must = 1")]
    InvalidGenesisHeight,
    #[error("Genesis account data deserialization")]
    GenesisAccountDeserialization,
    #[error("Error inserting into merkle tree")]
    TreeInsertionError,
    #[error("AppHash is null")]
    NullApphash,
    #[error("Invalid genesis app hash")]
    InvalidGenesisAppHash,
    #[error("Error looking up value")]
    TreeGetError,
    #[error("Sender balance not initialized")]
    SenderNotInitialized,
    #[error("Could not revert. Tree may be corrupted.")]
    CouldNotRevert,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("State root update returned None")]
    NoRoot,
    #[error("Could not serialize pairs")]
    SerializePairsError,
    #[error("Could not deserialize pairs")]
    DeserializePairsError,
}