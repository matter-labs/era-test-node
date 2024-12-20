use zksync_types::{Address, U256, U64};
use zksync_web3_decl::error::Web3Error;

use crate::{
    fork::ForkSource,
    namespaces::{HardhatNamespaceT, ResetRequest, RpcResult},
    node::InMemoryNode,
    utils::{into_jsrpc_error, into_jsrpc_error_message, IntoBoxedFuture},
};

impl<S: ForkSource + std::fmt::Debug + Clone + Send + Sync + 'static> HardhatNamespaceT
    for InMemoryNode<S>
{
    fn set_balance(&self, address: Address, balance: U256) -> RpcResult<bool> {
        self.set_balance(address, balance)
            .map_err(|err| {
                tracing::error!("failed setting balance : {:?}", err);
                into_jsrpc_error(Web3Error::InternalError(err))
            })
            .into_boxed_future()
    }

    fn set_nonce(&self, address: Address, balance: U256) -> RpcResult<bool> {
        self.set_nonce(address, balance)
            .map_err(|err| {
                tracing::error!("failed setting nonce: {:?}", err);
                into_jsrpc_error(Web3Error::InternalError(err))
            })
            .into_boxed_future()
    }

    fn hardhat_mine(&self, num_blocks: Option<U64>, interval: Option<U64>) -> RpcResult<()> {
        self.mine_blocks(num_blocks, interval)
            .map_err(|err| {
                tracing::error!("failed mining blocks: {:?}", err);
                into_jsrpc_error(Web3Error::InternalError(err))
            })
            .into_boxed_future()
    }

    fn hardhat_get_automine(&self) -> RpcResult<bool> {
        self.get_automine()
            .map_err(|err| {
                tracing::error!("failed getting automine: {:?}", err);
                into_jsrpc_error(Web3Error::InternalError(err))
            })
            .into_boxed_future()
    }

    fn reset_network(&self, reset_spec: Option<ResetRequest>) -> RpcResult<bool> {
        self.reset_network(reset_spec)
            .map_err(|err| {
                tracing::error!("failed reset: {:?}", err);
                into_jsrpc_error(Web3Error::InternalError(err))
            })
            .into_boxed_future()
    }

    fn impersonate_account(&self, address: Address) -> RpcResult<bool> {
        self.impersonate_account(address)
            .map_err(|err| {
                tracing::error!("failed impersonating account: {:?}", err);
                into_jsrpc_error(Web3Error::InternalError(err))
            })
            .into_boxed_future()
    }

    fn stop_impersonating_account(&self, address: Address) -> RpcResult<bool> {
        InMemoryNode::<S>::stop_impersonating_account(self, address)
            .map_err(|err| {
                tracing::error!("failed stopping to impersonate account: {:?}", err);
                into_jsrpc_error(Web3Error::InternalError(err))
            })
            .into_boxed_future()
    }

    fn set_code(&self, address: Address, code: String) -> RpcResult<()> {
        self.set_code(address, code)
            .map_err(|err| {
                tracing::error!("failed setting code: {:?}", err);
                into_jsrpc_error_message(err.to_string())
            })
            .into_boxed_future()
    }

    fn set_storage_at(&self, address: Address, slot: U256, value: U256) -> RpcResult<bool> {
        self.set_storage_at(address, slot, value)
            .map_err(|err| {
                tracing::error!("failed setting storage: {:?}", err);
                into_jsrpc_error(Web3Error::InternalError(err))
            })
            .into_boxed_future()
    }
}
