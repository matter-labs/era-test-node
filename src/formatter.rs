//! Helper methods to display transaction data in more human readable way.
use crate::{resolver, ShowCalls};

use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;

use zksync_basic_types::H160;
use zksync_contracts::read_file_to_json_value;
use zksync_core::block_on;

use zksync_types::{vm_trace::Call, VmEvent};

use lazy_static::lazy_static;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum ContractType {
    System,
    Precompile,
    Popular,
    Unknown,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KnownAddress {
    address: H160,
    name: String,
    contract_type: ContractType,
}

lazy_static! {
    /// Loads the known contact addresses from the JSON file.
    static ref KNOWN_ADDRESSES: HashMap<H160, KnownAddress> = {
        let json_value = read_file_to_json_value("core/bin/test_node/data/address_map.json");
        let pairs: Vec<KnownAddress> = serde_json::from_value(json_value).unwrap();

        pairs
            .into_iter()
            .map(|entry| (entry.address, entry))
            .collect()
    };
}

fn address_to_human_readable(address: H160) -> Option<String> {
    KNOWN_ADDRESSES
        .get(&address)
        .map(|known_address| match known_address.contract_type {
            ContractType::System => known_address.name.to_string(),
            ContractType::Precompile => format!("{}", known_address.name.dimmed()),
            ContractType::Popular => format!("{}", known_address.name.green()),
            ContractType::Unknown => known_address.name.to_string(),
        })
}

/// Pretty-prints event object
/// if skip_resolve is false, will try to contact openchain to resolve the topic hashes.
pub fn print_event(event: &VmEvent, skip_resolve: bool) {
    let event = event.clone();
    block_on(async move {
        let mut tt: Vec<String> = vec![];
        if skip_resolve {
            tt = event.indexed_topics.iter().map(|t| t.to_string()).collect();
        } else {
            for topic in event.indexed_topics {
                let selector = resolver::decode_event_selector(&format!(
                    "0x{}",
                    hex::encode(topic.as_bytes())
                ))
                .await
                .unwrap();
                tt.push(selector.unwrap_or(topic.to_string()));
            }
        }

        println!(
            "{} {}",
            address_to_human_readable(event.address)
                .map(|x| format!("{:42}", x))
                .unwrap_or(format!("{:?}", event.address)),
            tt.join(", ")
        );
    });
}

/// Pretty-prints contents of a 'call' - including subcalls.
/// If skip_resolve is false, will try to contact openchain to resolve the ABI names.
pub fn print_call(call: &Call, padding: usize, show_calls: &ShowCalls, skip_resolve: bool) {
    let contract_type = KNOWN_ADDRESSES
        .get(&call.to)
        .cloned()
        .map(|known_address| known_address.contract_type)
        .unwrap_or(ContractType::Unknown);

    let should_print = match (&contract_type, &show_calls) {
        (_, ShowCalls::All) => true,
        (_, ShowCalls::None) => false,
        // now we're left only with 'user' and 'system'
        (ContractType::Unknown, _) => true,
        (ContractType::Popular, _) => true,
        (ContractType::Precompile, _) => false,
        // Now we're left with System
        (ContractType::System, ShowCalls::User) => false,
        (ContractType::System, ShowCalls::System) => true,
    };
    if should_print {
        let function_signature = if call.input.len() >= 4 {
            let sig = call.input.as_slice()[..4]
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<Vec<_>>()
                .join("");

            if contract_type == ContractType::Precompile || skip_resolve {
                sig
            } else {
                block_on(async move {
                    let fetch = resolver::decode_function_selector(&sig).await.unwrap();
                    fetch.unwrap_or(format!("{}", format!("0x{}", sig).dimmed()))
                })
            }
        } else {
            format!(
                "0x{}",
                call.input
                    .as_slice()
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<Vec<_>>()
                    .join("")
            )
        };

        println!(
            "{}{:?} {} {} {} {} {}",
            " ".repeat(padding),
            call.r#type,
            address_to_human_readable(call.to)
                .map(|x| format!("{:42}", x))
                .unwrap_or(format!("{}", format!("{:?}", call.to).bold())),
            function_signature,
            call.revert_reason
                .as_ref()
                .map(|s| format!("Revert: {}", s.red()))
                .unwrap_or("".to_string()),
            call.error
                .as_ref()
                .map(|s| format!("Error: {}", s.red()))
                .unwrap_or("".to_string()),
            call.gas
        );
    }
    for subcall in &call.calls {
        print_call(subcall, padding + 2, show_calls, skip_resolve);
    }
}
