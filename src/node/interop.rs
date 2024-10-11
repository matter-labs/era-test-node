use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use zksync_multivm::interface::VmEvent;
use zksync_types::L2ChainId;
use zksync_types::H256;
use zksync_utils::h256_to_u256;

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
