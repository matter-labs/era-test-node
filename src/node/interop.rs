use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;
use zksync_multivm::interface::VmEvent;
use zksync_types::L2ChainId;
use zksync_types::H160;
use zksync_types::H256;
use zksync_types::U256;
use zksync_utils::h256_to_u256;

use super::InMemoryNode;
use crate::fork::ForkSource;

const LOCK_FILE: &str = "/tmp/interop/LOCK";

// InteropMessage(uint256,address,address,bytes)
pub const INTEROP_EVENT_HASH: &str =
    "aeb45e9fa7465a0054db321a0901056bc5e2ac40d10855aaaef37227d896635c";

#[derive(Serialize, Deserialize)]
pub struct InteropMessage {
    pub source_chain: u64,
    pub destination_chain: H256,
    pub destination_address: H256,
    pub source_address: H256,
    pub payload: String,
}

impl InteropMessage {
    const MINT_VAL_SLOT: usize = 4;
    const L2_VALUE_SLOT: usize = 5;
    const GAS_LIMIT_SLOT: usize = 6;
    const PUBDATA_LIMIT_SLOT: usize = 7;
    const REFUND_SLOT: usize = 8;

    const CALLDATA_LENGTH_SLOT: usize = 12;

    pub fn get_calldata(&self) -> Vec<u8> {
        let calldata_len = self.get_payload_word(Self::CALLDATA_LENGTH_SLOT).as_u64();
        let bytes_left = self.payload.len() - (Self::CALLDATA_LENGTH_SLOT + 1) * 64;
        assert_eq!(calldata_len, bytes_left as u64);

        hex::decode(&self.payload[64 * (Self::CALLDATA_LENGTH_SLOT + 1)..]).unwrap()
    }
    pub fn get_gas_limit(&self) -> U256 {
        self.get_payload_word(Self::GAS_LIMIT_SLOT)
    }
    pub fn get_pubdata_limit(&self) -> U256 {
        self.get_payload_word(Self::PUBDATA_LIMIT_SLOT)
    }

    pub fn get_mint_value(&self) -> U256 {
        self.get_payload_word(Self::MINT_VAL_SLOT)
    }

    pub fn get_l2_value(&self) -> U256 {
        self.get_payload_word(Self::L2_VALUE_SLOT)
    }

    pub fn canonical_tx_hash(&self) -> H256 {
        // FIXME
        H256::random()
    }

    pub fn get_refund_recipient(&self) -> H160 {
        let mut bytes = [0u8; 32];
        self.get_payload_word(Self::REFUND_SLOT)
            .to_big_endian(&mut bytes);
        H160::from_slice(&bytes[12..32])
    }

    fn get_payload_word(&self, slot_id: usize) -> U256 {
        let bytes = hex::decode(&self.payload[64 * slot_id..64 * (slot_id + 1)]).unwrap();
        U256::from_big_endian(&bytes)
    }

    pub fn compute_aliased_sender(&self) -> H160 {
        let source_chain = U256::from(self.source_chain);
        let mut hasher = Keccak256::new();
        hasher.update(self.source_address.as_bytes());
        let mut bytes = [0u8; 32];
        source_chain.to_big_endian(&mut bytes);
        hasher.update(bytes);
        let result = hasher.finalize();
        H160::from_slice(&result[0..20])
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct InteropMessages {
    messages: Vec<InteropMessage>,
}

pub fn send_interop(current_chain_id: L2ChainId, event: &VmEvent) {
    let lock = acquire_lock(LOCK_FILE);
    let destination_chain_id = event.indexed_topics.get(1).unwrap();

    let dst_file = format!(
        "/tmp/interop/interop_to_{}.json",
        h256_to_u256(*destination_chain_id).as_u64()
    );

    let mut previous_messages: InteropMessages = if let Ok(mut file) = File::open(dst_file.clone())
    {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read the file");
        serde_json::from_str(&contents).unwrap()
    } else {
        InteropMessages::default()
    };

    previous_messages.messages.push(InteropMessage {
        source_chain: current_chain_id.as_u64(),
        destination_chain: *destination_chain_id,
        destination_address: *event.indexed_topics.get(2).unwrap(),
        source_address: *event.indexed_topics.get(3).unwrap(),
        payload: hex::encode(event.value.clone()),
    });
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dst_file.clone())
        .expect(&format!("Unable to open the file for writing {}", dst_file));
    let updated_json =
        serde_json::to_string_pretty(&previous_messages).expect("Unable to serialize JSON");
    file.write_all(updated_json.as_bytes())
        .expect("Unable to write data");

    release_lock(&lock);
}

/// Acquire a lock by locking the lock file
pub fn acquire_lock(lock_file_path: &str) -> std::fs::File {
    let lock_path = Path::new(lock_file_path);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
        .expect("Unable to open lock file");

    file.lock_exclusive().expect("Unable to acquire lock");
    file
}

/// Release the lock by unlocking the lock file
pub fn release_lock(lock_file: &std::fs::File) {
    lock_file.unlock().expect("Unable to release lock");
}

pub struct InteropWatcher {}

impl InteropWatcher {
    pub async fn start_watching<
        S: std::marker::Sync + std::marker::Send + 'static + ForkSource + std::fmt::Debug + Clone,
    >(
        chain_id: u64,
        node: InMemoryNode<S>,
    ) -> tokio::task::JoinHandle<()> {
        let file_to_watch = format!("/tmp/interop/interop_to_{}.json", chain_id);

        println!("Watching file {} for changes...", file_to_watch);

        let mut interval = tokio::time::interval(Duration::from_secs(1));

        let mut messages_parsed = 0;

        // Run the file watcher in a tokio task
        tokio::spawn(async move {
            // Listen for file events
            loop {
                interval.tick().await;
                let path = Path::new(&file_to_watch);
                let previous_messages: InteropMessages = if let Ok(mut file) = File::open(path) {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)
                        .expect("Unable to read the file");
                    serde_json::from_str(&contents).unwrap()
                } else {
                    InteropMessages::default()
                };

                if previous_messages.messages.len() != messages_parsed {
                    println!(
                        "Seeing {} new messages",
                        previous_messages.messages.len() - messages_parsed
                    );

                    for message_id in messages_parsed..previous_messages.messages.len() {
                        let message = previous_messages.messages.get(message_id).unwrap();
                        node.run_interop(message).unwrap();
                    }

                    messages_parsed = previous_messages.messages.len();
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::node::interop::InteropMessage;

    #[test]
    fn test_get_calldata() {
        let input: &str = r#"    {
            "source_chain": 260,
            "destination_chain": "0x0000000000000000000000000000000000000000000000000000000000002f5b",
            "destination_address": "0x000000000000000000000000009e4886247aebebe2c5e5f6d11181ac33ddd7d4",
            "source_address": "0x000000000000000000000000e2b8cb53a43a56d4d2ab6131c81bd76b86d3afe5",
            "payload": "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000002f5b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002710000000000000000000000000000000000000000000000000000000000000c3500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009e4886247aebebe2c5e5f6d11181ac33ddd7d4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000000"
        }"#;
        let msg: InteropMessage = serde_json::from_str(input).unwrap();
        assert_eq!(0, msg.get_calldata().len());
        assert_eq!(10000, msg.get_gas_limit().as_u64());
    }
}
