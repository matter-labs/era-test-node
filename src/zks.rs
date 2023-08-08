use std::{
    convert::TryInto,
    sync::{Arc, RwLock},
};

use bigdecimal::BigDecimal;
use chrono::Utc;
use zksync_basic_types::{
    web3::signing::keccak256, L1BatchNumber, MiniblockNumber, H160, H256, U256,
};
use zksync_contracts::BaseSystemContractsHashes;
use zksync_core::api_server::web3::backend_jsonrpc::namespaces::zks::ZksNamespaceT;
use zksync_types::{
    api::{BridgeAddresses, TransactionDetails},
    block,
    explorer_api::{BlockDetails, BlockStatus, L1BatchDetails},
};

use crate::node::{InMemoryNodeInner, FAIR_L2_GAS_PRICE, MAX_TX_SIZE};

/// Mock implementation of ZksNamespace - used only in the test node.
pub struct ZkMockNamespaceImpl {
    pub inner_node: Arc<RwLock<InMemoryNodeInner>>,
    pub bridge_address: H160,
}

macro_rules! not_implemented {
    () => {
        Box::pin(async move { Err(jsonrpc_core::Error::method_not_found()) })
    };
}

impl ZksNamespaceT for ZkMockNamespaceImpl {
    /// We have to support this method, as zksync foundry depends on it.
    /// For now, returning a fake amount of gas.
    fn estimate_fee(
        &self,
        _req: zksync_types::transaction_request::CallRequest,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<zksync_types::fee::Fee>> {
        Box::pin(async move {
            Ok(zksync_types::fee::Fee {
                gas_limit: U256::from(1000000000),
                max_fee_per_gas: U256::from(1000000000),
                max_priority_fee_per_gas: U256::from(1000000000),
                gas_per_pubdata_limit: U256::from(1000000000),
            })
        })
    }

    fn get_raw_block_transactions(
        &self,
        _block_number: MiniblockNumber,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Vec<zksync_types::Transaction>>> {
        not_implemented!()
    }

    fn estimate_gas_l1_to_l2(
        &self,
        _req: zksync_types::transaction_request::CallRequest,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<U256>> {
        not_implemented!()
    }

    fn get_main_contract(
        &self,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<zksync_basic_types::Address>> {
        not_implemented!()
    }

    fn get_testnet_paymaster(
        &self,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Option<zksync_basic_types::Address>>> {
        not_implemented!()
    }

    fn get_bridge_contracts(
        &self,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<BridgeAddresses>> {
        let bridge_address = self.bridge_address.clone();
        Box::pin(async move {
            Ok(BridgeAddresses {
                l1_erc20_default_bridge: bridge_address.clone(),
                l2_erc20_default_bridge: bridge_address.clone(),
                l1_weth_bridge: None,
                l2_weth_bridge: None,
            })
        })
    }

    fn l1_chain_id(
        &self,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<zksync_basic_types::U64>> {
        not_implemented!()
    }

    fn get_confirmed_tokens(
        &self,
        _from: u32,
        _limit: u8,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Vec<zksync_web3_decl::types::Token>>> {
        not_implemented!()
    }

    fn get_token_price(
        &self,
        _token_address: zksync_basic_types::Address,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<BigDecimal>> {
        not_implemented!()
    }

    fn get_all_account_balances(
        &self,
        _address: zksync_basic_types::Address,
    ) -> jsonrpc_core::BoxFuture<
        jsonrpc_core::Result<std::collections::HashMap<zksync_basic_types::Address, U256>>,
    > {
        not_implemented!()
    }

    fn get_l2_to_l1_msg_proof(
        &self,
        _block: zksync_basic_types::MiniblockNumber,
        _sender: zksync_basic_types::Address,
        _msg: zksync_basic_types::H256,
        _l2_log_position: Option<usize>,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Option<zksync_types::api::L2ToL1LogProof>>>
    {
        not_implemented!()
    }

    fn get_l2_to_l1_log_proof(
        &self,
        _tx_hash: zksync_basic_types::H256,
        _index: Option<usize>,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Option<zksync_types::api::L2ToL1LogProof>>>
    {
        not_implemented!()
    }

    fn get_l1_batch_number(
        &self,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<zksync_basic_types::U64>> {
        not_implemented!()
    }

    fn get_block_details(
        &self,
        block_number: zksync_basic_types::MiniblockNumber,
    ) -> jsonrpc_core::BoxFuture<
        jsonrpc_core::Result<Option<zksync_types::explorer_api::BlockDetails>>,
    > {
        let reader = self.inner_node.read().unwrap();

        let block_details = reader
            .blocks
            .get(&block_number.0)
            .map(|block_info| BlockDetails {
                number: block_number,
                l1_batch_number: L1BatchNumber(block_number.0),
                timestamp: block_info.block_timestamp,
                l1_tx_count: 0,
                l2_tx_count: 1,
                root_hash: None,
                status: BlockStatus::Sealed,
                commit_tx_hash: None,
                committed_at: None,
                prove_tx_hash: None,
                proven_at: None,
                execute_tx_hash: None,
                executed_at: None,
                l1_gas_price: reader.l1_gas_price,
                l2_fair_gas_price: FAIR_L2_GAS_PRICE,
                base_system_contracts_hashes: reader.baseline_contracts.hashes(),
                operator_address: H160::zero(),
            });

        Box::pin(async move { Ok(block_details) })
    }

    fn get_miniblock_range(
        &self,
        _batch: zksync_basic_types::L1BatchNumber,
    ) -> jsonrpc_core::BoxFuture<
        jsonrpc_core::Result<Option<(zksync_basic_types::U64, zksync_basic_types::U64)>>,
    > {
        not_implemented!()
    }

    fn set_known_bytecode(
        &self,
        _bytecode: zksync_basic_types::Bytes,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<bool>> {
        not_implemented!()
    }

    fn get_transaction_details(
        &self,
        hash: zksync_basic_types::H256,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Option<zksync_types::api::TransactionDetails>>>
    {
        let reader = self.inner_node.read().unwrap();

        let transaction_details =
            reader
                .tx_results
                .get(&hash)
                .map(|tx_execution| TransactionDetails {
                    is_l1_originated: false,
                    status: zksync_types::explorer_api::TransactionStatus::Included,
                    fee: U256::from(155),
                    gas_per_pubdata: Some(U256::from(9800)),
                    initiator_address: tx_execution.tx.initiator_account(),
                    received_at: Utc::now(), //from(tx_execution.tx.received_timestamp_ms),
                    eth_commit_tx_hash: None,
                    eth_prove_tx_hash: None,
                    eth_execute_tx_hash: None,
                });

        Box::pin(async move { Ok(transaction_details) })
    }

    fn get_l1_batch_details(
        &self,
        batch: zksync_basic_types::L1BatchNumber,
    ) -> jsonrpc_core::BoxFuture<
        jsonrpc_core::Result<Option<zksync_types::explorer_api::L1BatchDetails>>,
    > {
        let reader = self.inner_node.read().unwrap();

        let l1_block_details = reader.blocks.get(&batch).map(|block_info| L1BatchDetails {
            number: L1BatchNumber(block_info.batch_number),
            timestamp: block_info.block_timestamp,
            // No l1_tx support in in-memory node for now.
            l1_tx_count: 0,
            // We put only single transaction per batch.
            l2_tx_count: 1,
            root_hash: None,
            status: BlockStatus::Sealed,
            commit_tx_hash: Some(H256(keccak256(
                format!("commit {}", block_info.batch_number).as_bytes(),
            ))),
            committed_at: Some(Utc::now()),
            prove_tx_hash: None,
            proven_at: None,
            execute_tx_hash: None,
            executed_at: None,
            l1_gas_price: reader.l1_gas_price,
            l2_fair_gas_price: FAIR_L2_GAS_PRICE,
            base_system_contracts_hashes: reader.baseline_contracts.hashes(),
        });

        Box::pin(async move { Ok(l1_block_details) })
    }

    fn get_bytecode_by_hash(
        &self,
        _hash: zksync_basic_types::H256,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<Option<Vec<u8>>>> {
        not_implemented!()
    }

    fn get_l1_gas_price(
        &self,
    ) -> jsonrpc_core::BoxFuture<jsonrpc_core::Result<zksync_basic_types::U64>> {
        not_implemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zksync_types::transaction_request::CallRequest;

    #[tokio::test]
    async fn test_estimate_fee() {
        let namespace = ZkMockNamespaceImpl;

        let mock_request = CallRequest {
            from: Some(
                "0x0000000000000000000000000000000000000000"
                    .parse()
                    .unwrap(),
            ),
            to: Some(
                "0x0000000000000000000000000000000000000001"
                    .parse()
                    .unwrap(),
            ),
            gas: Some(U256::from(21000)),
            gas_price: Some(U256::from(20)),
            max_fee_per_gas: Some(U256::from(30)),
            max_priority_fee_per_gas: Some(U256::from(10)),
            value: Some(U256::from(1000)),
            data: Some(vec![1, 2, 3, 4].into()),
            nonce: Some(U256::from(1)),
            transaction_type: Some(zksync_basic_types::U64::from(1)),
            access_list: None,
            eip712_meta: None,
        };

        let result = namespace.estimate_fee(mock_request).await.unwrap();

        assert_eq!(result.gas_limit, U256::from(1000000000));
        assert_eq!(result.max_fee_per_gas, U256::from(1000000000));
        assert_eq!(result.max_priority_fee_per_gas, U256::from(1000000000));
        assert_eq!(result.gas_per_pubdata_limit, U256::from(1000000000));
    }
}
