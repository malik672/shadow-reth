//! Shadow Reth: An open-source reth node with support for shadow bytecode.
//!
//! Works by using [`shadow-reth-exex`] to replay canonical transactions with shadow bytecode,
//! and [`shadow-reth-rpc`] to provide an RPC interface for interacting with shadow data.

use eyre::Result;
use reth_node_ethereum::EthereumNode;
use shadow_reth_exex::ShadowExEx;

fn main() -> Result<()> {
    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    reth::cli::Cli::parse_args().run(|builder, _| async move {
        let handle = builder
            .node(EthereumNode::default())
            .install_exex("ShadowExEx", ShadowExEx::init)
            .launch()
            .await?;

        handle.wait_for_node_exit().await
    })
}