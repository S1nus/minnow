use ed25519_dalek::{
    VerifyingKey,
    SigningKey,
    Signature,
    Signer,
    Verifier,
};

use crate::errors::EasyFraudError;

// total size: 72 bytes
#[derive(Debug)]
pub struct Transaction {
    pub sender_pubkey: [u8; 32],
    pub recipient_pubkey: [u8; 32],
    pub amount: u64,
}

// total size: 136 bytes
#[derive(Debug)]
pub struct SignedTransaction {
    pub transaction_data: [u8; 72],
    pub signature: [u8; 64],
}

impl Transaction {

    pub fn sign(&self, signing_key: SigningKey) -> SignedTransaction {
        let serialized = self.serialize();
        let signature = signing_key.sign(&serialized[..]);
        SignedTransaction { 
            transaction_data: serialized, 
            signature: signature.to_bytes(), 
        }
    }

    // we're rolling our own share-aware serialization!
    pub fn serialize(&self) -> [u8; 72] {
        let mut buf = [0u8; 72];
        buf[..32].copy_from_slice(&self.sender_pubkey[..]);
        buf[32..64].copy_from_slice(&self.recipient_pubkey[..]);
        buf[64..72].copy_from_slice(&self.amount.to_le_bytes()[..]);
        buf
    }

    pub fn deserialize(bytes: [u8; 72]) -> Result<Self, EasyFraudError> {
        Ok(Transaction {
            sender_pubkey: bytes[..32].try_into()
                .map_err(|_| EasyFraudError::TransactionDeserializationError)?,
            recipient_pubkey: bytes[32..64].try_into()
                .map_err(|_| EasyFraudError::TransactionDeserializationError)?,
            amount: u64::from_le_bytes(bytes[64..].try_into().map_err(|_| EasyFraudError::TransactionDeserializationError)?),

        })
    }
}

impl SignedTransaction {
    pub fn serialize(&self) -> [u8; 136] {
        let mut buf = [0u8; 136];
        buf[..72].copy_from_slice(&self.transaction_data[..]);
        buf[72..].copy_from_slice(&self.signature[..]);
        buf
    }

    pub fn deserialize(bytes: [u8; 72]) -> Result<Self, EasyFraudError> {
        Ok(SignedTransaction { 
            transaction_data: bytes[..72].try_into()
                .map_err(|_| EasyFraudError::TransactionDeserializationError)?,
            signature: bytes[72..].try_into()
                .map_err(|_| EasyFraudError::TransactionDeserializationError)?,
        })
    }

    pub fn verify(&self) -> Result<bool, EasyFraudError> {
        let txn = Transaction::deserialize(self.transaction_data)?;
        let vk = VerifyingKey::from_bytes(&txn.sender_pubkey)
            .map_err(|_| EasyFraudError::TransactionDeserializationError)?;
        let sig = Signature::from_bytes(&self.signature);
        Ok(vk.verify(&self.transaction_data, &sig).is_ok())
    }

    pub fn verify_and_deserialize(&self) -> Result<Transaction, EasyFraudError> {
        let txn = Transaction::deserialize(self.transaction_data)?;
        let vk = VerifyingKey::from_bytes(&txn.sender_pubkey)
            .map_err(|_| EasyFraudError::TransactionDeserializationError)?;
        let sig = Signature::from_bytes(&self.signature);
        if vk.verify(&self.transaction_data, &sig).is_ok() {
            return Ok(txn);
        }
        Err(EasyFraudError::InvalidSignature)
    }
}