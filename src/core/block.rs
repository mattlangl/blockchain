use std::{io::{self, Write, Read, Cursor}};

use chrono::Utc;
use encode_decode_derive::{Encode, Decode};
use p256::ecdsa::Signature;
use sha2::{Sha256, Digest};
use crate::{types::hash::Hash, crypto::keypair::{PublicKey, PrivateKey}};

use super::{transaction::{Transaction}, encoding::{Encoder, Decoder, Encode, Decode, HeaderEncoder}};

#[derive(Debug, PartialEq, Eq, Encode, Decode, Clone, Copy)]
pub struct Header {
    pub version: u32,
    pub data: Option<Hash>,
    pub prev_block: Hash,
    pub timestamp: i64,
    pub height: u32,
}


#[derive(Debug, PartialEq, Decode, Encode, Clone)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
    pub signature: Option<Signature>,
    pub validator: Option<PublicKey>,
    pub hash: Hash, // Cached version of the header hash
}


impl Block {
    pub fn new(header: Header, transactions: Vec<Transaction>) -> Block {
        let mut buf = Vec::new();
        let encoder = HeaderEncoder::new();
        header.encode_binary(&mut buf, encoder).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(buf);
        let fin = hasher.finalize().to_vec();
        Block {
            header,
            transactions,
            hash: Hash::from_bytes(&fin).expect("can't convert"),
            signature: None,
            validator: None,
        }
    }


    pub fn random_block(h: u32) -> Self {
        let header = Header {
            version: 1,
            data: None,
            prev_block: Hash::random(),
            timestamp: Utc::now().timestamp(),
            height: h,
        };
        let tx = Transaction {
            data: br#"foo"#.to_vec(),
            key: None,
            signature: None,
        };

        Block::new(header, vec![tx])
    }

    pub fn random_block_with_signature(h: u32) -> Self {
        let key = PrivateKey::generate_key();
        let mut b = Self::random_block(h);
        assert!(b.sign(key).is_ok());
    
        return b
    }
    

    pub fn hash(&mut self) -> Hash {
        if self.hash.is_zero() {
            let mut buf = Vec::new();
            let encoder = HeaderEncoder::new();
            self.header.encode_binary(&mut buf, encoder).unwrap();
            let mut hasher = Sha256::new();
            hasher.update(buf);
            let fin = hasher.finalize().to_vec();
            self.hash = Hash::from_bytes(&fin).expect("failed");
        }

        self.hash
    }

    pub fn header_data(&self) -> Result<Vec<u8>, io::Error> {
        let encoder = HeaderEncoder::new();
        let mut vec = Cursor::new(vec![]);
        self.header.encode_binary(&mut vec, encoder).expect("couldn't encode header");
        Ok(vec.get_ref().to_owned())
    }

    pub fn sign(&mut self, key: PrivateKey) -> Result<(), String> {
        let header = self.header_data().unwrap();
        self.signature = Some(key.sign(&header).expect("could not sign"));
        self.validator = Some(key.generate_public());
        Ok(())
    }

    pub fn verify(&self) -> Result<(), String> {
        if self.signature.is_none() {
            return Err("no signature".to_string());
        }

        let validator = self.validator.as_ref().unwrap();
        let signature = self.signature.unwrap();
        let res = validator.verify(&self.header_data().unwrap(), &signature);
        if res.is_err() {
            return Err("Could not verify".to_owned());
        }
        Ok(())
    }


}


#[cfg(test)]
mod test {
    // use crate::types::hash::Hash;

    // use super::*;
    // use std::io::Cursor;

    // #[test]
    // fn test_header_encode_decode() {
    //     let h = Header {
    //         version: 1,
    //         prev_block: Hash::random(),
    //         timestamp: chrono::Utc::now().timestamp_nanos(),
    //         height: 10,
    //         nonce: 989394,
    //     };

    //     let mut buf = Cursor::new(vec![]);
    //     assert!(h.encode_binary(&mut buf).is_ok());

    //     buf.set_position(0);
    //     let h_decode = Header::decode_binary(&mut buf).unwrap();
    //     assert_eq!(h, h_decode);
    // }

    // #[test]
    // fn test_block_encode_decode() {
    //     let header = Header {
    //         version: 1,
    //         prev_block: Hash::random(),
    //         timestamp: chrono::Utc::now().timestamp_nanos(),
    //         height: 10,
    //         nonce: 989394,
    //     };
    //     let b = Block::new(header, vec![]);

    //     let mut buf = Cursor::new(vec![]);
    //     assert!(b.encode_binary(&mut buf).is_ok());

    //     buf.set_position(0);
    //     let b_decode = Block::decode_binary(&mut buf).unwrap();
    //     println!("third part");
    //     assert_eq!(b, b_decode);
    // }

    // #[test]
    // fn test_block_hash() {
    //     let mut b = Block {
    //         header: Header {
    //             version: 1,
    //             prev_block: Hash::random(),
    //             timestamp: chrono::Utc::now().timestamp_nanos(),
    //             height: 10,
    //             nonce: 0,
    //         },
    //         transactions: vec![],
    //         hash: Hash::default(),
    //     };

    //     let h = b.hash();
    //     assert!(!h.is_zero());
    // }

    use std::time::{self, SystemTime};

    use crate::{crypto::{self, keypair::PrivateKey}, types::hash::Hash, core::transaction::Transaction};

    use super::{Block, Header};

    use chrono::{Utc};


    #[test]
    fn test_sign_block() {
        let key = PrivateKey::generate_key();
        let mut b = Block::random_block(0);
        assert!(b.sign(key).is_ok());
        assert!(b.signature.is_some());
    }

    #[test]
    fn test_verify_block() {
        let key = PrivateKey::generate_key();
        let mut b = Block::random_block(0);
        assert!(b.sign(key).is_ok());
        assert!(b.verify().is_ok());

        let other_key = PrivateKey::generate_key();
        let old_validator = b.validator;
        b.validator = Some(other_key.generate_public());
        assert!(b.verify().is_err());
        b.validator = old_validator;
        b.header.height = 100;

        assert!(b.verify().is_err());

    }

}