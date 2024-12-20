use crate::namespaces::Result;
use crate::observability::LogLevel;
use jsonrpc_derive::rpc;

#[rpc]
pub trait ConfigurationApiNamespaceT {
    /// Get the InMemoryNodeInner's show_calls property as a string
    ///
    /// # Returns
    /// The current `show_calls` value for the InMemoryNodeInner.
    #[rpc(name = "config_getShowCalls", returns = "String")]
    fn config_get_show_calls(&self) -> Result<String>;

    /// Get the InMemoryNodeInner's show_outputs property as a boolean
    ///
    /// # Returns
    /// The current `show_outputs` value for the InMemoryNodeInner.
    #[rpc(name = "config_getShowOutputs", returns = "bool")]
    fn config_get_show_outputs(&self) -> Result<bool>;

    /// Get the InMemoryNodeInner's current_timestamp property
    ///
    /// # Returns
    /// The current `current_timestamp` value for the InMemoryNodeInner.
    #[rpc(name = "config_getCurrentTimestamp", returns = "u64")]
    fn config_get_current_timestamp(&self) -> Result<u64>;

    /// Set show_calls for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A ShowCalls enum to update show_calls to
    ///
    /// # Returns
    /// The updated/current `show_calls` value for the InMemoryNodeInner.
    #[rpc(name = "config_setShowCalls", returns = "String")]
    fn config_set_show_calls(&self, value: String) -> Result<String>;

    /// Set show_outputs for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: a bool value to update show_outputs to
    ///
    /// # Returns
    /// The updated/current `show_outputs` value for the InMemoryNodeInner.
    #[rpc(name = "config_setShowOutputs", returns = "bool")]
    fn config_set_show_outputs(&self, value: bool) -> Result<bool>;

    /// Set show_storage_logs for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A ShowStorageLogs enum to update show_storage_logs to
    ///
    /// # Returns
    /// The updated/current `show_storage_logs` value for the InMemoryNodeInner.
    #[rpc(name = "config_setShowStorageLogs", returns = "String")]
    fn config_set_show_storage_logs(&self, value: String) -> Result<String>;

    /// Set show_vm_details for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A ShowVMDetails enum to update show_vm_details to
    ///
    /// # Returns
    /// The updated/current `show_vm_details` value for the InMemoryNodeInner.
    #[rpc(name = "config_setShowVmDetails", returns = "String")]
    fn config_set_show_vm_details(&self, value: String) -> Result<String>;

    /// Set show_gas_details for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A ShowGasDetails enum to update show_gas_details to
    ///
    /// # Returns
    /// The updated/current `show_gas_details` value for the InMemoryNodeInner.
    #[rpc(name = "config_setShowGasDetails", returns = "String")]
    fn config_set_show_gas_details(&self, value: String) -> Result<String>;

    /// Set resolve_hashes for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A bool to update resolve_hashes to
    ///
    /// # Returns
    /// The updated `resolve_hashes` value for the InMemoryNodeInner.
    #[rpc(name = "config_setResolveHashes", returns = "bool")]
    fn config_set_resolve_hashes(&self, value: bool) -> Result<bool>;

    /// Set show_node_config for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A bool to update show_node_config to
    ///
    /// # Returns
    /// The updated/current `show_node_config` value for the InMemoryNodeInner.
    #[rpc(name = "config_setShowNodeConfig", returns = "bool")]
    fn config_set_show_node_config(&self, value: bool) -> Result<bool>;

    /// Set show_tx_summary for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A bool to update show_tx_summary to
    ///
    /// # Returns
    /// The updated/current `show_tx_summary` value for the InMemoryNodeInner.
    #[rpc(name = "config_setShowTxSummary", returns = "bool")]
    fn config_set_show_tx_summary(&self, value: bool) -> Result<bool>;

    /// Set show_event_logs for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A bool to update show_event_logs to
    ///
    /// # Returns
    /// The updated/current `show_event_logs` value for the InMemoryNodeInner.
    #[rpc(name = "config_setShowEventLogs", returns = "bool")]
    fn config_set_show_event_logs(&self, value: bool) -> Result<bool>;

    /// Set disable_console_log for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `value`: A bool to update disable_console_log to
    ///
    /// # Returns
    /// The updated/current `disable_console_log` value for the InMemoryNodeInner.
    #[rpc(name = "config_setDisableConsoleLog", returns = "bool")]
    fn config_set_disable_console_log(&self, value: bool) -> Result<bool>;

    /// Set the logging for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `level`: The log level to set. One of: ["trace", "debug", "info", "warn", "error"]
    ///
    /// # Returns
    /// `true` if the operation succeeded, `false` otherwise.
    #[rpc(name = "config_setLogLevel", returns = "bool")]
    fn config_set_log_level(&self, level: LogLevel) -> Result<bool>;

    /// Set the logging for the InMemoryNodeInner
    ///
    /// # Parameters
    /// - `level`: The logging directive to set. Example:
    ///     * "my_crate=debug"
    ///     * "my_crate::module=trace"
    ///     * "my_crate=debug,other_crate=warn"
    ///
    /// # Returns
    /// `true` if the operation succeeded, `false` otherwise.
    #[rpc(name = "config_setLogging", returns = "bool")]
    fn config_set_logging(&self, directive: String) -> Result<bool>;
}
