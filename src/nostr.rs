use secp256k1::{Secp256k1, SecretKey, PublicKey};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: String,
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u32,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl Event {
    pub fn new(secret_key: &SecretKey, content: String, kind: u32, tags: Vec<Vec<String>>) -> Self {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, secret_key);
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut event = Event {
            id: String::new(),
            pubkey: hex::encode(&public_key.serialize()[1..33]),
            created_at,
            kind,
            tags,
            content,
            sig: String::new(),
        };

        event.id = event.calculate_id();
        event.sig = event.sign(secret_key);
        
        event
    }

    fn calculate_id(&self) -> String {
        let serialized = json!([
            0,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content
        ]);

        let mut hasher = Sha256::new();
        hasher.update(serialized.to_string().as_bytes());
        hex::encode(hasher.finalize())
    }

    fn sign(&self, secret_key: &SecretKey) -> String {
        let secp = Secp256k1::new();
        let message = secp256k1::Message::from_slice(
            &hex::decode(&self.id).unwrap()
        ).unwrap();
        
        let keypair = secp256k1::KeyPair::from_secret_key(&secp, secret_key);
        let sig = secp.sign_schnorr_no_aux_rand(&message, &keypair);
        hex::encode(sig.as_ref())
    }
}