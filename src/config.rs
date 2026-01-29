//! 环境与网络配置：集中管理 TRON/EVM 网络选择、示例地址、RPC URL
//!
//! 环境变量说明：
//! - TRON_NETWORK: nile | mainnet | shasta（默认 nile）
//! - EVM_NETWORK: sepolia | arbitrum-sepolia | arbitrum-one | mainnet（默认 sepolia）
//! - EVM_RPC_URL: 覆盖 EVM RPC，未设置时从 SDK 备选列表健康检查选取

use chains_sdk::rpc::chains::evm::{EvmNetwork, EvmRpcProvider};
use chains_sdk::rpc::chains::tron::TronNetwork;
use chains_sdk::rpc::RpcProvider;
use std::env;
use std::time::Duration;

/// Tron 示例地址（Nile 测试网，与 run_verify.py 一致）
pub const TRON_EXAMPLE_ADDR: &str = "TG2D8vTp4xHBB2vhVbgHK2AhA2p9wY4q9M";

/// EVM 示例地址（Sepolia，0x 格式，与 run_verify.py 一致）
pub const EVM_EXAMPLE_ADDR: &str = "0x3CCD11B6c4B5Ca62d2B29C949B23e0550d64f0b9";

/// 从环境变量 TRON_NETWORK 解析当前 Tron 网络（nile | mainnet | shasta），默认 nile
pub fn current_tron_network() -> TronNetwork {
    match env::var("TRON_NETWORK").as_deref() {
        Ok("mainnet") => TronNetwork::Mainnet,
        Ok("shasta") => TronNetwork::Shasta,
        _ => TronNetwork::Nile,
    }
}

/// 从环境变量 EVM_NETWORK 解析当前 EVM 网络，默认 sepolia
pub fn current_evm_network() -> EvmNetwork {
    match env::var("EVM_NETWORK").as_deref() {
        Ok("arbitrum-sepolia") => EvmNetwork::ArbitrumSepolia,
        Ok("arbitrum-one") => EvmNetwork::ArbitrumOne,
        Ok("mainnet") => EvmNetwork::Mainnet,
        _ => EvmNetwork::Sepolia,
    }
}

/// 未设置 EVM_RPC_URL 时，从 SDK 备选列表中选第一个可用的 RPC（单次健康检查 8 秒超时）
pub async fn evm_rpc_url(network: EvmNetwork) -> String {
    if let Ok(url) = env::var("EVM_RPC_URL") {
        return url;
    }
    for url in network.urls() {
        let p = EvmRpcProvider::new((*url).to_string());
        match tokio::time::timeout(Duration::from_secs(8), p.health_check()).await {
            Ok(Ok(true)) => return url.to_string(),
            _ => {}
        }
    }
    network.url().to_string()
}
