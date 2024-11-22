//! Helper methods to display transaction data in more human readable way.
use crate::{config::show_details::ShowCalls, resolver};

use colored::Colorize;

use serde::Deserialize;
use std::collections::HashMap;
use std::str;

use crate::fork::block_on;
use crate::utils::{calculate_eth_cost, format_gwei, to_human_size};
use futures::future::join_all;
use lazy_static::lazy_static;
use zksync_multivm::interface::{Call, VmEvent, VmExecutionResultAndLogs};
use zksync_types::Address;
use zksync_types::{StorageLogWithPreviousValue, Transaction};
use zksync_types::{H160, H256};

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
        let json_value = serde_json::from_slice(include_bytes!("data/address_map.json")).unwrap();
        let pairs: Vec<KnownAddress> = serde_json::from_value(json_value).unwrap();

        pairs
            .into_iter()
            .map(|entry| (entry.address, entry))
            .collect()
    };
}

pub struct Formatter {
    sibling_stack: Vec<bool>,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            sibling_stack: Vec::new(),
        }
    }

    pub fn with_initial_stack(sibling_stack: Vec<bool>) -> Self {
        Formatter { sibling_stack }
    }

    pub fn section<F>(&mut self, title: &str, has_more_siblings: bool, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.format_log(false, title);
        self.enter_scope(has_more_siblings);
        f(self);
        self.exit_scope();
    }

    pub fn subsection<F>(&mut self, title: &str, has_more_siblings: bool, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.format_log(false, title);
        self.enter_scope(has_more_siblings);
        f(self);
        self.exit_scope();
    }

    pub fn item(&mut self, is_last_sibling: bool, key: &str, value: &str) {
        self.format_log(
            is_last_sibling,
            &format!("{}: {}", key.bold(), value.dimmed()),
        );
    }

    pub fn warning(&mut self, is_last_sibling: bool, message: &str) {
        self.format_error(is_last_sibling, &format!("WARNING: {}", message));
    }

    pub fn enter_scope(&mut self, has_more_siblings: bool) {
        self.sibling_stack.push(has_more_siblings);
    }

    pub fn exit_scope(&mut self) {
        self.sibling_stack.pop();
    }

    pub fn format_log(&self, is_last_sibling: bool, message: &str) {
        let prefix = build_prefix(&self.sibling_stack, is_last_sibling);
        tracing::info!("{}{}", prefix, message);
    }

    pub fn format_error(&self, is_last_sibling: bool, message: &str) {
        let prefix = build_prefix(&self.sibling_stack, is_last_sibling);
        tracing::info!("{}", format!("{}{}", prefix, message).red());
    }
}

fn build_prefix(sibling_stack: &[bool], is_last_sibling: bool) -> String {
    let mut prefix = String::new();
    let depth = sibling_stack.len();
    if depth > 0 {
        for &has_more_siblings in &sibling_stack[..depth - 1] {
            if has_more_siblings {
                prefix.push_str("│   ");
            } else {
                prefix.push_str("    ");
            }
        }
        let branch = if is_last_sibling {
            "└─ "
        } else {
            "├─ "
        };
        prefix.push_str(branch);
    }
    prefix
}

// TODO: When refactoring other logs (e.g event, storage, vm, gas) update this function.
// Currently a close duplicate of format_address_human_readable
fn address_to_human_readable(address: H160) -> Option<String> {
    KNOWN_ADDRESSES.get(&address).map(|known_address| {
        let name = match known_address.contract_type {
            ContractType::System => known_address.name.bold().bright_blue().to_string(),
            ContractType::Precompile => known_address.name.bold().magenta().to_string(),
            ContractType::Popular => known_address.name.bold().bright_green().to_string(),
            ContractType::Unknown => known_address.name.dimmed().to_string(),
        };

        let formatted_address = format!("{:#x}", address).dimmed();
        format!("{}{}{}", name, "@".dimmed(), formatted_address)
    })
}

fn format_address_human_readable(
    address: H160,
    initiator: H160,
    contract_address: Option<H160>,
    call_type: &str,
) -> Option<String> {
    let is_initiator = address == initiator;
    let excluded_addresses = [
        H160::from_slice(&hex::decode("0000000000000000000000000000000000008006").unwrap()),
        H160::from_slice(&hex::decode("0000000000000000000000000000000000010000").unwrap()),
    ];

    let is_contract = Some(address) == contract_address && !excluded_addresses.contains(&address);

    if is_initiator {
        let name = "initiator".bold().green().to_string();
        let formatted_address = format!("{:#x}", address);
        return Some(format!("{}{}{}", name, "@".dimmed(), formatted_address));
    }
    if call_type == "Create" {
        let name = "deployed".bold().bright_green().to_string();
        let formatted_address = format!("{:#x}", address);
        return Some(format!("{}{}{}", name, "@".dimmed(), formatted_address));
    } else if is_contract {
        let name = "contract".bold().bright_green().to_string();
        let formatted_address = format!("{:#x}", address);
        return Some(format!("{}{}{}", name, "@".dimmed(), formatted_address));
    }

    KNOWN_ADDRESSES.get(&address).map(|known_address| {
        let name = match known_address.contract_type {
            ContractType::System => known_address.name.bold().bright_blue().to_string(),
            ContractType::Precompile => known_address.name.bold().magenta().to_string(),
            ContractType::Popular => known_address.name.bold().bright_green().to_string(),
            ContractType::Unknown => known_address.name.dimmed().to_string(),
        };

        let formatted_address = format!("{:#x}", address).dimmed();
        format!("{}{}{}", name, "@".dimmed(), formatted_address)
    })
}

fn resolve_topics(topics: &[H256], resolve_hashes: bool) -> Vec<String> {
    let topics = topics.to_owned();
    block_on(async move {
        let futures = topics.into_iter().map(|topic| async move {
            if resolve_hashes {
                resolver::decode_event_selector(&format!("{:#x}", topic))
                    .await
                    .unwrap_or(None)
                    .unwrap_or_else(|| format!("{:#x}", topic))
            } else {
                format!("{:#x}", topic)
            }
        });

        join_all(futures).await
    })
}

pub fn print_event(event: &VmEvent, resolve_hashes: bool, is_last_sibling: bool) {
    // Clone the event to avoid borrowing issues
    let event = event.clone();
    let mut formatter = Formatter::new();

    // Resolve topics before logging
    let topics = resolve_topics(&event.indexed_topics, resolve_hashes);

    // Format the event address (contract)
    let contract_display = address_to_human_readable(event.address)
        .map(|x| format!("{:42}", x.blue()))
        .unwrap_or_else(|| format!("{:42}", format!("{:?}", event.address).blue()));

    // Start the event section
    formatter.section(
        &format!("Event [{}]", contract_display),
        !is_last_sibling,
        |event_section| {
            // We'll always have two subitems: Topics and Data
            let total_subitems = 2;
            let mut subitem_idx = 0;

            // Topics Section
            subitem_idx += 1;
            let is_last_subitem = subitem_idx == total_subitems;

            event_section.section("Topics", !is_last_subitem, |topics_section| {
                let num_topics = topics.len();
                if num_topics == 0 {
                    topics_section.item(true, "Topics", "EMPTY");
                } else {
                    for (i, topic) in topics.iter().enumerate() {
                        let is_last_topic = i == num_topics - 1;
                        topics_section.item(is_last_topic, &format!("Topic[{}]", i), topic);
                    }
                }
            });

            // Data Section
            subitem_idx += 1;
            let is_last_subitem = subitem_idx == total_subitems;

            if event.value.is_empty() {
                event_section.item(is_last_subitem, "Data", "EMPTY");
            } else {
                let data_str = match str::from_utf8(&event.value) {
                    Ok(v) => format!("{}", v.truecolor(128, 128, 128)),
                    Err(_) => {
                        let hex_str = hex::encode(&event.value);
                        let display_str = if hex_str.len() > 200 {
                            format!("{}...", &hex_str[..200])
                        } else {
                            hex_str
                        };
                        format!("0x{}", display_str.truecolor(128, 128, 128))
                    }
                };
                event_section.item(is_last_subitem, "Data", &data_str);
            }
        },
    );
}

/// Amount of pubdata that given write has cost.
pub enum PubdataBytesInfo {
    // This slot is free
    FreeSlot,
    // This slot costs this much.
    Paid(u32),
    // This happens when we already paid a little for this slot in the past.
    // This slots costs additional X, the total cost is Y.
    AdditionalPayment(u32, u32),
    // We already paid for this slot in this transaction.
    PaidAlready,
}

impl std::fmt::Display for PubdataBytesInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PubdataBytesInfo::FreeSlot => write!(f, "Free Slot (no cost)"),
            PubdataBytesInfo::Paid(cost) => {
                write!(f, "Paid: {} bytes", to_human_size((*cost).into()))
            }
            PubdataBytesInfo::AdditionalPayment(additional_cost, total_cost) => write!(
                f,
                "Additional Payment: {} bytes (Total: {} bytes)",
                to_human_size((*additional_cost).into()),
                to_human_size((*total_cost).into())
            ),
            PubdataBytesInfo::PaidAlready => write!(f, "Already Paid (no additional cost)"),
        }
    }
}

impl PubdataBytesInfo {
    // Whether the slot incurs any cost
    pub fn does_cost(&self) -> bool {
        match self {
            PubdataBytesInfo::FreeSlot => false,
            PubdataBytesInfo::Paid(_) => true,
            PubdataBytesInfo::AdditionalPayment(_, _) => true,
            PubdataBytesInfo::PaidAlready => false,
        }
    }
}

pub fn print_call(
    initiator: Address,
    contract_address: Option<H160>,
    call: &Call,
    formatter: &mut Formatter,
    is_last_sibling: bool,
    show_calls: &ShowCalls,
    show_outputs: bool,
    resolve_hashes: bool,
) {
    let contract_type = KNOWN_ADDRESSES
        .get(&call.to)
        .cloned()
        .map(|known_address| known_address.contract_type)
        .unwrap_or(ContractType::Unknown);

    let should_print = match (&contract_type, &show_calls) {
        (_, ShowCalls::All) => true,
        (_, ShowCalls::None) => false,
        (ContractType::Unknown, _) => true,
        (ContractType::Popular, _) => true,
        (ContractType::Precompile, _) => false,
        (ContractType::System, ShowCalls::User) => false,
        (ContractType::System, ShowCalls::System) => true,
    };

    if should_print {
        let call_type_display = format!("{:?}", call.r#type).blue();
        let remaining_gas_display = to_human_size(call.gas.into()).yellow();
        let gas_used_display = format!("({})", to_human_size(call.gas_used.into())).bold();

        // Get contract display
        let contract_display = format_address_human_readable(
            call.to,
            initiator,
            contract_address,
            format!("{:?}", call.r#type).as_str(),
        )
        .unwrap_or_else(|| format!("{:}", format!("{:?}", call.to).bold()));

        // Get function signature
        let function_signature = if call.input.len() >= 4 {
            let sig = hex::encode(&call.input[0..4]);
            if contract_type == ContractType::Precompile || !resolve_hashes {
                format!("0x{}", sig)
            } else {
                block_on(async move {
                    match resolver::decode_function_selector(&sig).await {
                        Ok(Some(name)) => name,
                        Ok(None) | Err(_) => format!("0x{}", sig),
                    }
                })
            }
        } else {
            "unknown".to_string()
        };

        let function_display = function_signature.cyan().bold();

        // Build the line
        let line = format!(
            "{} [{}] {}::{} {}",
            call_type_display,
            remaining_gas_display,
            contract_display,
            function_display,
            gas_used_display
        );

        // Handle errors
        if call.revert_reason.is_some() || call.error.is_some() {
            formatter.format_error(is_last_sibling, &line);
            if let Some(ref reason) = call.revert_reason {
                formatter.enter_scope(!is_last_sibling);
                formatter.format_error(true, &format!("🔴 Revert reason: {}", reason));
                formatter.exit_scope();
            }
            if let Some(ref error) = call.error {
                formatter.enter_scope(!is_last_sibling);
                formatter.format_error(true, &format!("🔴 Error: {}", error));
                formatter.exit_scope();
            }
        } else {
            formatter.format_log(is_last_sibling, &line);
        }

        // Handle outputs
        if show_outputs && !call.output.is_empty() {
            let output_display = call
                .output
                .as_slice()
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<Vec<_>>()
                .join("");

            formatter.enter_scope(!is_last_sibling);
            formatter.format_log(true, &format!("Output: {}", output_display.dimmed()));
            formatter.exit_scope();
        }
    }

    // Process subcalls
    if !call.calls.is_empty() {
        formatter.enter_scope(!is_last_sibling);

        let num_subcalls = call.calls.len();
        for (i, subcall) in call.calls.iter().enumerate() {
            let is_last_subcall = i == num_subcalls - 1;
            print_call(
                initiator,
                contract_address,
                subcall,
                formatter,
                is_last_subcall,
                show_calls,
                show_outputs,
                resolve_hashes,
            );
        }

        formatter.exit_scope();
    }
}

pub fn print_logs(
    log_query: &StorageLogWithPreviousValue,
    pubdata_bytes: Option<PubdataBytesInfo>,
    log_index: usize,
    formatter: &mut Formatter,
    is_last: bool,
) {
    formatter.format_log(is_last, &format!("Log #{}", log_index));

    formatter.enter_scope(!is_last);
    formatter.format_log(false, &format!("Kind:           {:?}", log_query.log.kind));
    formatter.format_log(
        false,
        &format!(
            "Address:        {}",
            address_to_human_readable(*log_query.log.key.address())
                .unwrap_or_else(|| format!("{}", log_query.log.key.address()))
        ),
    );
    formatter.format_log(
        false,
        &format!("Key:            {:#066x}", log_query.log.key.key()),
    );
    formatter.format_log(
        false,
        &format!("Read Value:     {:#066x}", log_query.previous_value),
    );

    if log_query.log.is_write() {
        formatter.format_log(
            false,
            &format!("Written Value:  {:#066x}", log_query.log.value),
        );
    }

    if let Some(pubdata_bytes) = pubdata_bytes {
        formatter.format_log(true, &format!("Pubdata Bytes:  {}", pubdata_bytes));
    } else {
        formatter.format_log(true, "Pubdata Bytes:  None");
    }
    formatter.exit_scope();
}

pub fn print_vm_details2(result: &VmExecutionResultAndLogs) {
    tracing::info!("");
    tracing::info!("[VM Execution Results]");

    let mut formatter = Formatter::new();

    // Log the main statistics
    formatter.format_log(
        false,
        &format!(
            "Cycles Used:          {}",
            to_human_size(result.statistics.cycles_used.into())
        ),
    );
    formatter.format_log(
        false,
        &format!(
            "Computation Gas Used: {}",
            to_human_size(result.statistics.computational_gas_used.into())
        ),
    );
    formatter.format_log(
        false,
        &format!(
            "Contracts Used:       {}",
            to_human_size(result.statistics.contracts_used.into())
        ),
    );

    // Log execution outcome
    match &result.result {
        zksync_multivm::interface::ExecutionResult::Success { .. } => {
            formatter.format_log(true, "Execution Outcome:    Success");
        }
        zksync_multivm::interface::ExecutionResult::Revert { output } => {
            formatter.format_log(false, "Execution Outcome:    Failure");
            formatter.enter_scope(true);
            formatter.format_error(
                true,
                &format!("Revert Reason:    {}", output.to_user_friendly_string()),
            );
            formatter.exit_scope();
        }
        zksync_multivm::interface::ExecutionResult::Halt { reason } => {
            formatter.format_log(false, "Execution Outcome:    Failure");
            formatter.enter_scope(true);
            formatter.format_error(true, &format!("Halt Reason:      {}", reason.to_string()));
            formatter.exit_scope();
        }
    }

    tracing::info!("");
}

pub fn print_transaction_summary(
    l2_gas_price: u64,
    tx: &Transaction,
    tx_result: &VmExecutionResultAndLogs,
    status: &str,
) {
    // Calculate used and refunded gas
    let used_gas = tx.gas_limit() - tx_result.refunds.gas_refunded;
    let paid_in_eth = calculate_eth_cost(l2_gas_price, used_gas.as_u64());

    let refunded_gas = tx_result.refunds.gas_refunded;

    // Calculate refunded gas in ETH
    let refunded_in_eth = calculate_eth_cost(l2_gas_price, refunded_gas);

    let emoji = match status {
        "SUCCESS" => "✅",
        "FAILED" => "❌",
        "HALTED" => "⏸️",
        _ => "⚠️",
    };

    tracing::info!("{}  [{}] Hash: {:?}", emoji, status, tx.hash());
    tracing::info!("Initiator: {:?}", tx.initiator_account());
    tracing::info!("Payer: {:?}", tx.payer());
    tracing::info!(
        "Gas Usage: Limit: {} | Used: {} | Refunded: {}",
        to_human_size(tx.gas_limit()),
        to_human_size(used_gas),
        to_human_size(tx_result.refunds.gas_refunded.into())
    );
    tracing::info!(
        "Paid: {:.10} ETH ({} gas * {})",
        paid_in_eth,
        used_gas,
        format_gwei(l2_gas_price.into())
    );
    tracing::info!("Refunded: {:.10} ETH", refunded_in_eth);
}

pub fn _print_vm_details(result: &VmExecutionResultAndLogs) {
    tracing::info!("");
    tracing::info!("┌──────────────────────────┐");
    tracing::info!("│   VM EXECUTION RESULTS   │");
    tracing::info!("└──────────────────────────┘");

    tracing::info!("Cycles Used:          {}", result.statistics.cycles_used);
    tracing::info!(
        "Computation Gas Used: {}",
        result.statistics.computational_gas_used
    );
    tracing::info!("Contracts Used:       {}", result.statistics.contracts_used);
    match &result.result {
        zksync_multivm::interface::ExecutionResult::Success { .. } => {}
        zksync_multivm::interface::ExecutionResult::Revert { output } => {
            tracing::info!("");
            tracing::info!(
                "{}",
                format!(
                    "\n[!] Revert Reason:    {}",
                    output.to_user_friendly_string()
                )
                .on_red()
            );
        }
        zksync_multivm::interface::ExecutionResult::Halt { reason } => {
            tracing::info!("");
            tracing::info!("{}", format!("\n[!] Halt Reason:    {}", reason).on_red());
        }
    }

    tracing::info!("════════════════════════════");
}
