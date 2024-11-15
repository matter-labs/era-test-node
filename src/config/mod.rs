use std::{env, fs::read_to_string, path::PathBuf};

use crate::{observability, system_contracts};

use clap::{Parser, ValueEnum};
use cli::Cli;
use observability::LogLevel;
use serde::Deserialize;
use std::{fmt::Display, str::FromStr};

pub mod cli;

/// Directory where configuration files are stored
pub const CONFIG_DIR: &str = ".era_test_node";
/// Default name of the configuration file
pub const CONFIG_FILE_NAME: &str = "config.toml";
/// Default directory for disk cache
pub const DEFAULT_DISK_CACHE_DIR: &str = ".cache";
/// Default L1 gas price for transactions
pub const DEFAULT_L1_GAS_PRICE: u64 = 14_932_364_075;
/// Default L2 gas price for transactions if not provided via CLI
pub const DEFAULT_L2_GAS_PRICE: u64 = 45_250_000;
/// Default price for fair pubdata based on predefined value
pub const DEFAULT_FAIR_PUBDATA_PRICE: u64 = 13_607_659_111;
/// Scale factor for estimating L1 gas prices
pub const DEFAULT_ESTIMATE_GAS_PRICE_SCALE_FACTOR: f64 = 2.0;
/// Scale factor for estimating gas limits
pub const DEFAULT_ESTIMATE_GAS_SCALE_FACTOR: f32 = 1.3;
/// Default port for the test node server
pub const NODE_PORT: u16 = 8011;
/// Network ID for the test node
pub const TEST_NODE_NETWORK_ID: u32 = 260;
/// Default log file path for the test node
pub const DEFAULT_LOG_FILE_PATH: &str = "era_test_node.log";

/// Defines the configuration parameters for the [InMemoryNode].
#[derive(Deserialize, Debug, Clone)]
pub struct TestNodeConfig {
    /// Port the node will listen on
    pub port: u16,
    /// Controls visibility of call logs
    pub show_calls: ShowCalls,
    /// Whether to show call output data
    pub show_outputs: bool,
    /// Level of detail for storage logs
    pub show_storage_logs: ShowStorageLogs,
    /// Level of detail for VM execution logs
    pub show_vm_details: ShowVMDetails,
    /// Level of detail for gas usage logs
    pub show_gas_details: ShowGasDetails,
    /// Whether to resolve hash references
    pub resolve_hashes: bool,
    /// Configuration for system contracts
    pub system_contracts_options: system_contracts::Options,
    /// Enables EVM emulation mode
    pub use_evm_emulator: bool,
    /// Optional chain ID for the node
    pub chain_id: Option<u32>,
    /// L1 gas price (optional override)
    pub l1_gas_price: Option<u64>,
    /// L2 gas price (optional override)
    pub l2_gas_price: Option<u64>,
    /// Price for pubdata on L1
    pub l1_pubdata_price: Option<u64>,
    /// L1 gas price scale factor for gas estimation
    pub price_scale_factor: Option<f64>,
    /// The factor by which to scale the gasLimit
    pub limit_scale_factor: Option<f32>,
    /// Logging verbosity level
    pub log_level: LogLevel,
    /// Path to the log file
    pub log_file_path: String,
    /// Cache configuration for the test node
    pub cache_config: CacheConfig,
}

impl Default for TestNodeConfig {
    fn default() -> Self {
        Self {
            // Node configuration defaults
            port: NODE_PORT,
            show_calls: Default::default(),
            show_outputs: false,
            show_storage_logs: Default::default(),
            show_vm_details: Default::default(),
            show_gas_details: Default::default(),
            resolve_hashes: false,
            system_contracts_options: Default::default(),
            use_evm_emulator: false,
            chain_id: Some(TEST_NODE_NETWORK_ID),

            // Gas configuration defaults
            l1_gas_price: Some(DEFAULT_L1_GAS_PRICE),
            l2_gas_price: Some(DEFAULT_L2_GAS_PRICE),
            l1_pubdata_price: Some(DEFAULT_FAIR_PUBDATA_PRICE),
            price_scale_factor: Some(DEFAULT_ESTIMATE_GAS_PRICE_SCALE_FACTOR),
            limit_scale_factor: Some(DEFAULT_ESTIMATE_GAS_SCALE_FACTOR),

            // Log configuration defaults
            log_level: Default::default(),
            log_file_path: String::from(DEFAULT_LOG_FILE_PATH),

            // Cache configuration default
            cache_config: Default::default(),
        }
    }
}

impl TestNodeConfig {
    /// Try to load a configuration file from either a provided path or the `$HOME` directory.
    pub fn try_load(file_path: &Option<String>) -> eyre::Result<TestNodeConfig> {
        let path = if let Some(path) = file_path {
            PathBuf::from(path)
        } else {
            // NOTE: `env::home_dir` is not compatible with Windows.
            #[allow(deprecated)]
            let mut path = env::home_dir().expect("failed to get home directory");

            path.push(CONFIG_DIR);
            path.push(CONFIG_FILE_NAME);
            path
        };

        let toml = read_to_string(path)?;
        let config = toml::from_str(&toml)?;

        Ok(config)
    }

    /// Set the port for the test node
    #[must_use]
    pub fn with_port(mut self, port: Option<u16>) -> Self {
        if let Some(port) = port {
            self.port = port;
        }
        self
    }

    /// Get the port for the test node
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Set the chain ID for the test node
    #[must_use]
    pub fn with_chain_id(mut self, chain_id: Option<u32>) -> Self {
        if let Some(chain_id) = chain_id {
            self.chain_id = Some(chain_id);
        }
        self
    }

    /// Get the chain ID for the test node
    pub fn get_chain_id(&self) -> u32 {
        self.chain_id.unwrap_or(TEST_NODE_NETWORK_ID)
    }

    /// Set the system contracts configuration option
    #[must_use]
    pub fn with_system_contracts(mut self, option: Option<system_contracts::Options>) -> Self {
        if let Some(option) = option {
            self.system_contracts_options = option;
        }
        self
    }

    /// Get the system contracts configuration option
    pub fn get_system_contracts(&self) -> system_contracts::Options {
        self.system_contracts_options
    }

    /// Enable or disable EVM emulation
    #[must_use]
    pub fn with_evm_emulator(mut self, enable: Option<bool>) -> Self {
        if let Some(enable) = enable {
            self.use_evm_emulator = enable;
        }
        self
    }

    /// Get the EVM emulation status
    pub fn is_evm_emulator_enabled(&self) -> bool {
        self.use_evm_emulator
    }

    /// Set the L1 gas price
    #[must_use]
    pub fn with_l1_gas_price(mut self, price: Option<u64>) -> Self {
        if let Some(price) = price {
            self.l1_gas_price = Some(price);
        }
        self
    }

    /// Get the L1 gas price
    pub fn get_l1_gas_price(&self) -> u64 {
        self.l1_gas_price.unwrap_or(DEFAULT_L1_GAS_PRICE)
    }

    /// Set the L2 gas price
    #[must_use]
    pub fn with_l2_gas_price(mut self, price: Option<u64>) -> Self {
        if let Some(price) = price {
            self.l2_gas_price = Some(price);
        }
        self
    }

    /// Get the L2 gas price
    pub fn get_l2_gas_price(&self) -> u64 {
        self.l2_gas_price.unwrap_or(DEFAULT_L2_GAS_PRICE)
    }

    /// Set the L1 pubdata price
    #[must_use]
    pub fn with_l1_pubdata_price(mut self, price: u64) -> Self {
        self.l1_pubdata_price = Some(price);
        self
    }

    /// Get the L1 pubdata price
    pub fn get_l1_pubdata_price(&self) -> u64 {
        self.l1_pubdata_price.unwrap_or(DEFAULT_FAIR_PUBDATA_PRICE)
    }

    /// Set the log level
    #[must_use]
    pub fn with_log_level(mut self, level: Option<LogLevel>) -> Self {
        if let Some(level) = level {
            self.log_level = level;
        }
        self
    }

    /// Get the log level
    pub fn get_log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Set the cache configuration
    #[must_use]
    pub fn with_cache_config(mut self, config: Option<CacheConfig>) -> Self {
        if let Some(config) = config {
            self.cache_config = config;
        }
        self
    }

    /// Get the cache configuration
    pub fn get_cache_config(&self) -> &CacheConfig {
        &self.cache_config
    }

    /// Set the log file path
    #[must_use]
    pub fn with_log_file_path(mut self, path: Option<String>) -> Self {
        if let Some(path) = path {
            self.log_file_path = path;
        }
        self
    }

    /// Get the log file path
    pub fn get_log_file_path(&self) -> &str {
        &self.log_file_path
    }

    /// Set the visibility of call logs
    #[must_use]
    pub fn with_show_calls(mut self, show_calls: Option<ShowCalls>) -> Self {
        if let Some(show_calls) = show_calls {
            self.show_calls = show_calls;
        }
        self
    }

    /// Get the visibility of call logs
    pub fn get_show_calls(&self) -> ShowCalls {
        self.show_calls
    }

    /// Enable or disable resolving hashes
    #[must_use]
    pub fn with_resolve_hashes(mut self, resolve: Option<bool>) -> Self {
        if let Some(resolve) = resolve {
            self.resolve_hashes = resolve;
        }
        self
    }

    /// Check if resolving hashes is enabled
    pub fn is_resolve_hashes_enabled(&self) -> bool {
        self.resolve_hashes
    }

    /// Set the gas limit scale factor
    #[must_use]
    pub fn with_gas_limit_scale(mut self, scale: Option<f32>) -> Self {
        if let Some(scale) = scale {
            self.limit_scale_factor = Some(scale);
        }
        self
    }

    /// Get the gas limit scale factor
    pub fn get_gas_limit_scale(&self) -> f32 {
        self.limit_scale_factor
            .unwrap_or(DEFAULT_ESTIMATE_GAS_SCALE_FACTOR)
    }

    /// Set the price scale factor
    #[must_use]
    pub fn with_price_scale(mut self, scale: Option<f64>) -> Self {
        if let Some(scale) = scale {
            self.price_scale_factor = Some(scale);
        }
        self
    }

    /// Get the price scale factor
    pub fn get_price_scale(&self) -> f64 {
        self.price_scale_factor
            .unwrap_or(DEFAULT_ESTIMATE_GAS_PRICE_SCALE_FACTOR)
    }

    /// Set the detail level of VM execution logs
    #[must_use]
    pub fn with_vm_log_detail(mut self, detail: Option<ShowVMDetails>) -> Self {
        if let Some(detail) = detail {
            self.show_vm_details = detail;
        }
        self
    }

    /// Get the detail level of VM execution logs
    pub fn get_vm_log_detail(&self) -> ShowVMDetails {
        self.show_vm_details
    }

    // TODO: do we need this?
    /// Override the config with values provided by [`Cli`].
    pub fn override_with_opts(&mut self, opt: &Cli) {
        // [`NodeConfig`].
        if let Some(port) = &opt.port {
            self.port = *port;
        }

        if opt.debug_mode {
            self.show_calls = ShowCalls::All;
            self.show_outputs = true;
            self.show_gas_details = ShowGasDetails::All;
            self.resolve_hashes = true;
        }
        if let Some(show_calls) = &opt.show_calls {
            self.show_calls = *show_calls;
        }
        if let Some(show_outputs) = &opt.show_outputs {
            self.show_outputs = *show_outputs;
        }
        if let Some(show_storage_logs) = &opt.show_storage_logs {
            self.show_storage_logs = *show_storage_logs;
        }
        if let Some(show_vm_details) = &opt.show_vm_details {
            self.show_vm_details = *show_vm_details;
        }
        if let Some(show_gas_details) = &opt.show_gas_details {
            self.show_gas_details = *show_gas_details;
        }
        if let Some(resolve_hashes) = &opt.resolve_hashes {
            self.resolve_hashes = *resolve_hashes;
        }

        if opt.chain_id.is_some() {
            self.chain_id = opt.chain_id;
        }

        if let Some(contract_options) = opt.dev_system_contracts {
            self.system_contracts_options = contract_options;
        }

        if opt.emulate_evm {
            assert_eq!(
                self.system_contracts_options,
                system_contracts::Options::Local,
                "EVM emulation currently requires using local contracts"
            );
            self.use_evm_emulator = true;
        }

        // [`GasConfig`]
        if let Some(l1_gas_price) = &opt.l1_gas_price {
            self.l1_gas_price = Some(*l1_gas_price);
        }
        if let Some(l2_gas_price) = &opt.l2_gas_price {
            self.l2_gas_price = Some(*l2_gas_price);
        }

        // [`LogConfig`].
        if let Some(log_level) = &opt.log {
            self.log_level = *log_level;
        }
        if let Some(file_path) = &opt.log_file_path {
            self.log_file_path = file_path.to_string();
        }

        // [`CacheConfig`].
        if let Some(cache_type) = &opt.cache {
            self.cache_config = match cache_type {
                CacheType::None => CacheConfig::None,
                CacheType::Memory => CacheConfig::Memory,
                CacheType::Disk => CacheConfig::Disk {
                    dir: opt.cache_dir.clone().expect("missing --cache-dir argument"),
                    reset: opt.reset_cache.unwrap_or_default(),
                },
            };
        }
    }
}

#[derive(
    Deserialize, Debug, Default, clap::Parser, Copy, Clone, clap::ValueEnum, PartialEq, Eq,
)]
pub enum ShowCalls {
    #[default]
    None,
    User,
    System,
    All,
}

impl FromStr for ShowCalls {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "none" => Ok(ShowCalls::None),
            "user" => Ok(ShowCalls::User),
            "system" => Ok(ShowCalls::System),
            "all" => Ok(ShowCalls::All),
            _ => Err(format!(
                "Unknown ShowCalls value {} - expected one of none|user|system|all.",
                s
            )),
        }
    }
}

impl Display for ShowCalls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug, Default, Parser, Copy, Clone, clap::ValueEnum, PartialEq, Eq)]
pub enum ShowStorageLogs {
    #[default]
    None,
    Read,
    Write,
    Paid,
    All,
}

impl FromStr for ShowStorageLogs {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "none" => Ok(ShowStorageLogs::None),
            "read" => Ok(ShowStorageLogs::Read),
            "write" => Ok(ShowStorageLogs::Write),
            "paid" => Ok(ShowStorageLogs::Paid),
            "all" => Ok(ShowStorageLogs::All),
            _ => Err(format!(
                "Unknown ShowStorageLogs value {} - expected one of none|read|write|paid|all.",
                s
            )),
        }
    }
}

impl Display for ShowStorageLogs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug, Default, Parser, Copy, Clone, clap::ValueEnum, PartialEq, Eq)]
pub enum ShowVMDetails {
    #[default]
    None,
    All,
}

impl FromStr for ShowVMDetails {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "none" => Ok(ShowVMDetails::None),
            "all" => Ok(ShowVMDetails::All),
            _ => Err(format!(
                "Unknown ShowVMDetails value {} - expected one of none|all.",
                s
            )),
        }
    }
}

impl Display for ShowVMDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug, Default, Parser, Copy, Clone, clap::ValueEnum, PartialEq, Eq)]
pub enum ShowGasDetails {
    #[default]
    None,
    All,
}

impl FromStr for ShowGasDetails {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "none" => Ok(ShowGasDetails::None),
            "all" => Ok(ShowGasDetails::All),
            _ => Err(format!(
                "Unknown ShowGasDetails value {} - expected one of none|all.",
                s
            )),
        }
    }
}

impl Display for ShowGasDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

/// Cache type config for the node.
#[derive(ValueEnum, Deserialize, Default, Debug, Copy, Clone)]
pub enum CacheType {
    None,
    Memory,
    #[default]
    Disk,
}

/// Cache configuration. Can be one of:
///
/// None    : Caching is disabled
/// Memory  : Caching is provided in-memory and not persisted across runs
/// Disk    : Caching is persisted on disk in the provided directory and can be reset
#[derive(Deserialize, Debug, Clone)]
pub enum CacheConfig {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "memory")]
    Memory,
    #[serde(rename = "disk")]
    Disk { dir: String, reset: bool },
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::Disk {
            dir: String::from(DEFAULT_DISK_CACHE_DIR),
            reset: false,
        }
    }
}
