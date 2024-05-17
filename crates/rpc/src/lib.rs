//! ShadowRPC is a reth RPC extension which allows for reading
//! shadow data written to SQLite by [`reth-shadow-exex`]

/// Contains logic for custom RPC API methods.
pub(crate) mod apis;

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use reth::{
    providers::{
        test_utils::TestCanonStateSubscriptions, AccountReader, BlockNumReader, BlockReaderIdExt,
        ChainSpecProvider, ChangeSetReader, EvmEnvProvider, StateProviderFactory,
    },
    rpc::builder::{RethRpcModule, RpcModuleBuilder, RpcServerConfig, TransportRpcModuleConfig},
    tasks::TokioTaskExecutor,
};
use reth_node_ethereum::EthEvmConfig;
use shadow_reth_common::SqliteManager;

use self::apis::{GetLogsResponse, GetLogsRpcRequest};

#[rpc(server, namespace = "shadow")]
pub trait ShadowRpcApi {
    /// Returns shadow logs.
    #[method(name = "getLogs")]
    async fn get_logs(&self, req: GetLogsRpcRequest) -> RpcResult<GetLogsResponse>;
}

/// Wrapper around an RPC provider and a database connection pool.
#[derive(Debug)]
pub struct ShadowRpc<P> {
    provider: P,
    /// Database manager.
    sqlite_manager: SqliteManager,
}

impl<Provider> ShadowRpc<Provider>
where
    Provider: BlockNumReader + BlockReaderIdExt + Clone + Unpin,
{
    /// Instatiate a Shadow RPC API.
    pub async fn new(
        provider: Provider,
        db_path: &str,
    ) -> Result<ShadowRpc<Provider>, sqlx::Error> {
        Ok(Self { provider, sqlite_manager: SqliteManager::new(db_path).await? })
    }
}

/// Build an RPC server configuration extended with a custom namespace
/// containing methods for interacting with shadow data.
pub async fn build_rpc_server_config_with_shadow_namespace<Provider>(
    provider: Provider,
    db_path: &str,
) -> RpcServerConfig
where
    Provider: AccountReader
        + BlockNumReader
        + BlockReaderIdExt
        + ChainSpecProvider
        + ChangeSetReader
        + Clone
        + EvmEnvProvider
        + StateProviderFactory
        + Unpin
        + 'static,
{
    // TODO: Determine best way to add request verification middleware;
    // it should verify JSON-RPC version and ensure that the chosen method comes from an allowlist

    let rpc_builder = RpcModuleBuilder::default()
        .with_provider(provider.clone())
        // These are just dummy values to satisfy trait bounds
        .with_noop_pool()
        .with_noop_network()
        .with_executor(TokioTaskExecutor::default())
        .with_evm_config(EthEvmConfig::default())
        .with_events(TestCanonStateSubscriptions::default());

    let config = TransportRpcModuleConfig::default().with_http([RethRpcModule::Eth]);
    let mut server = rpc_builder.build(config);

    let shadow_rpc = ShadowRpc::new(provider, db_path).await.unwrap();
    server.merge_configured(shadow_rpc.into_rpc()).unwrap();

    RpcServerConfig::http(Default::default()).with_http_address("0.0.0.0:8545".parse().unwrap())
}

#[cfg(test)]
mod tests {
    use reth::providers::test_utils::MockEthProvider;
    use reth_primitives::{Block, Header};
    use shadow_reth_common::ShadowLog;

    use crate::{
        apis::{GetLogsParameters, GetLogsResponse, GetLogsResult, GetLogsRpcRequest},
        ShadowRpc, ShadowRpcApiServer,
    };

    const JSON_RPC_PROTOCOL_VERSION: &str = "2.0";

    #[tokio::test]
    async fn test_shadow_get_logs() {
        let mock_provider = MockEthProvider::default();

        let first_block = Block {
            header: Header { number: 18870000, ..Default::default() },
            ..Default::default()
        };
        let first_block_hash = first_block.hash_slow();

        let last_block = Block {
            header: Header { number: 18870001, ..Default::default() },
            ..Default::default()
        };
        let last_block_hash = last_block.hash_slow();

        mock_provider.extend_blocks([
            (first_block_hash, first_block.clone()),
            (last_block_hash, last_block.clone()),
        ]);

        let testing_db_path = std::env::temp_dir().join("test.db");

        let rpc = ShadowRpc::new(mock_provider, testing_db_path.to_str().unwrap()).await.unwrap();

        let logs = vec![
            ShadowLog {
                address: "0x0fbc0a9be1e87391ed2c7d2bb275bec02f53241f".to_string(),
                block_hash: "0x4131d538cf705c267da7f448ec7460b177f40d28115ad290ba6a1fd734afe280"
                    .to_string(),
                block_log_index: 0,
                block_number: 18870000,
                block_timestamp: 1703595263,
                transaction_index: 167,
                transction_hash: "0x8bf2361656e0ea6f338ad17ac3cd616f8eea9bb17e1afa1580802e9d3231c203"
                    .to_string(),
                transaction_log_index: 26,
                removed: false,
                data: Some("0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000049dc9ce34ad2a2177480000000000000000000000000000000000000000000000000432f754f7158ad80000000000000000000000000000000000000000000000000000000000000000".to_string()),
                topic_0: Some("0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822".to_string()),
                topic_1: Some("0x0000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad".to_string()),
                topic_2: Some("0x0000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad".to_string()),
                topic_3: None,
            },
            ShadowLog {
                address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string(),
                block_hash: "0x3cac643a6a1af584681a6a6dc632cd110a479c9c642e2da92b73fefb45739165".to_string(),
                block_log_index: 0,
                block_number: 18870001,
                block_timestamp: 1703595275,
                transaction_index: 2,
                transction_hash: "0xd02dc650cc9a34def3d7a78808a36a8cb2e292613c2989f4313155e8e4af9b0f".to_string(),
                transaction_log_index: 0,
                removed: false,
                data: Some("0x0000000000000000000000000000000000000000000000001bc16d674ec80000".to_string()),
                topic_0: Some("0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c".to_string()),
                topic_1: Some("0x0000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad".to_string()),
                topic_2: None,
                topic_3: None,
            },
        ];

        rpc.sqlite_manager.bulk_insert_into_shadow_log_table(logs).await.unwrap();

        let params = vec![GetLogsParameters {
            address: vec![
                "0x0fbc0a9be1e87391ed2c7d2bb275bec02f53241f".to_string(),
                "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string(),
                "0xc55126051b22ebb829d00368f4b12bde432de5da".to_string(),
            ],
            block_hash: None,
            from_block: Some("0x11feef0".to_string()),
            to_block: Some("0x11feef1".to_string()),
            topics: vec![],
        }];

        let req = GetLogsRpcRequest {
            id: "1".to_string(),
            json_rpc: JSON_RPC_PROTOCOL_VERSION.to_string(),
            method: "shadow_getLogs".to_string(),
            params,
        };

        let resp = rpc.get_logs(req.clone()).await.unwrap();

        let expected = GetLogsResponse {
            id: req.id,
            json_rpc: JSON_RPC_PROTOCOL_VERSION.to_string(),
            result: vec![
                GetLogsResult {
                    address: "0x0fbc0a9be1e87391ed2c7d2bb275bec02f53241f".to_string(),
                    block_hash: "0x4131d538cf705c267da7f448ec7460b177f40d28115ad290ba6a1fd734afe280".to_string(),
                    block_number: hex::encode(18870000u64.to_be_bytes()),
                    data: Some("0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000049dc9ce34ad2a2177480000000000000000000000000000000000000000000000000432f754f7158ad80000000000000000000000000000000000000000000000000000000000000000".to_string()),
                    log_index: 0u64.to_string(),
                    removed: false,
                    topics: [Some("0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822".to_string()), Some("0x0000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad".to_string()), Some("0x0000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad".to_string()), None],
                    transaction_hash: "0x8bf2361656e0ea6f338ad17ac3cd616f8eea9bb17e1afa1580802e9d3231c203".to_string(),
                    transaction_index: 167u64.to_string()
                },
                GetLogsResult {
                    address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string(),
                    block_hash: "0x3cac643a6a1af584681a6a6dc632cd110a479c9c642e2da92b73fefb45739165".to_string(),
                    block_number: hex::encode(18870001u64.to_be_bytes()),
                    data: Some("0x0000000000000000000000000000000000000000000000001bc16d674ec80000".to_string()),
                    log_index: 0u64.to_string(),
                    removed: false,
                    topics: [Some("0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c".to_string()), Some("0x0000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad".to_string()), None, None],
                    transaction_hash: "0xd02dc650cc9a34def3d7a78808a36a8cb2e292613c2989f4313155e8e4af9b0f".to_string(),
                    transaction_index: 2u64.to_string()
                }
            ],
        };

        assert_eq!(resp, expected);
        std::fs::remove_file(testing_db_path).unwrap()
    }
}
