//! ERC20 代币测试与示例（以太坊 / EVM）
//!
//! 只读查询、构建交易、全自动流程（构建 → 签名 → 广播 → 监听）

use crate::config;
use chains_sdk::chain::evm::{ethereum_address_from_private_key, sign_ethereum_transaction, EvmChain};
use chains_sdk::Blockchain;
use chains_sdk::rpc::chains::evm::{EvmNetwork, EvmRpcProvider};
use chains_sdk::transaction::{TransactionMonitor, TransactionSender, TransactionStatus};
use std::env;
use std::sync::Arc;

/// 监听 ETH / ERC20 交易确认（Sepolia）
///
/// 环境变量：
/// - TX_HASH：交易哈希（0x 格式，必填）
/// - EVM_RPC_URL：可选，默认 Sepolia 备选 RPC
/// - MONITOR_TIMEOUT_SEC：可选，超时秒数，默认 120
/// - MONITOR_MIN_CONFIRMATIONS：可选，最少确认数，默认 1
pub async fn run_eth_monitor() -> Result<(), Box<dyn std::error::Error>> {
    let tx_hash = match env::var("TX_HASH") {
        Ok(v) => v,
        Err(_) => {
            println!("请通过环境变量 TX_HASH 提供要监听的交易哈希（0x 格式）。");
            println!("例如：export TX_HASH=0x683ab7b8b1f8e2643f82e3351e48dee0b452a14ae4473dad90fc28ecacd84314");
            return Ok(());
        }
    };

    let network = config::current_evm_network();
    let rpc_url = config::evm_rpc_url(network).await;
    let provider = EvmRpcProvider::new(rpc_url.clone());
    let chain = EvmChain::new(chains_sdk::types::ChainType::Ethereum, Arc::new(provider));
    let monitor = TransactionMonitor::new(Arc::new(chain));

    let timeout_sec = env::var("MONITOR_TIMEOUT_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120);
    let min_confirmations = env::var("MONITOR_MIN_CONFIRMATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    println!("=== 监听 ETH/ERC20 交易（{}）===", network.name());
    println!("交易哈希: {}", tx_hash);
    println!("RPC:      {}", rpc_url);
    println!("超时:     {}s，最少确认: {}", timeout_sec, min_confirmations);
    println!();

    let status = monitor
        .wait_for_confirmation_with_timeout(&tx_hash, timeout_sec, Some(3000), Some(min_confirmations))
        .await?;

    match status {
        TransactionStatus::Confirmed => println!("✅ 交易已确认!"),
        TransactionStatus::Failed => println!("❌ 交易失败!"),
        TransactionStatus::Pending => println!("⏳ 超时仍未确认"),
    }

    Ok(())
}

/// 查询原生 ETH 余额（Sepolia 测试网）
///
/// 环境变量：
/// - ETH_ADDRESS：要查询的地址（0x 格式），未设置时使用示例地址
/// - EVM_RPC_URL：可选
pub async fn run_eth_balance() -> Result<(), Box<dyn std::error::Error>> {
    let network = config::current_evm_network();
    let rpc_url = config::evm_rpc_url(network).await;
    let address = env::var("ETH_ADDRESS").unwrap_or_else(|_| config::EVM_EXAMPLE_ADDR.to_string());

    let provider = EvmRpcProvider::new(rpc_url.clone());
    let chain = EvmChain::new(chains_sdk::types::ChainType::Ethereum, Arc::new(provider.clone()));

    println!("=== 原生 ETH 余额（{}）===", network.name());
    println!("地址: {}", address);
    println!("RPC:  {}", rpc_url);

    match chain.get_balance(&address).await {
        Ok(wei) => {
            let wei_u128: u128 = wei.parse::<u128>().unwrap_or(0);
            let eth = wei_u128 as f64 / 1e18;
            println!("余额(wei): {}", wei);
            println!("余额(ETH): {:.18}", eth);
        }
        Err(e) => println!("查询失败: {}", e),
    }

    Ok(())
}

/// 全自动原生 ETH 转账：构建 → 签名 → 广播 → 监听
///
/// 环境变量：
/// - ETH_PRIVATE_KEY：发送方私钥（64 位十六进制）
/// - ETH_FROM_ADDRESS、ETH_TO_ADDRESS：可选
/// - ETH_AMOUNT_WEI：转账金额（wei，字符串），默认 "1000000000000000"（0.001 ETH）
/// - EVM_RPC_URL：可选
pub async fn run_eth_transfer() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = match env::var("ETH_PRIVATE_KEY") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("未设置 ETH_PRIVATE_KEY。");
            eprintln!("示例: export ETH_PRIVATE_KEY=你的64位十六进制私钥");
            return Ok(());
        }
    };
    let network = config::current_evm_network();
    let rpc_url = config::evm_rpc_url(network).await;
    let from_addr = env::var("ETH_FROM_ADDRESS").unwrap_or_else(|_| config::EVM_EXAMPLE_ADDR.to_string());
    let to_addr = env::var("ETH_TO_ADDRESS").unwrap_or_else(|_| config::EVM_EXAMPLE_ADDR.to_string());
    let value_wei = env::var("ETH_AMOUNT_WEI").unwrap_or_else(|_| "1000000000000000".to_string());

    let provider = EvmRpcProvider::new(rpc_url.clone());
    let chain = EvmChain::new(chains_sdk::types::ChainType::Ethereum, Arc::new(provider.clone()));

    println!("=== 全自动原生 ETH 转账（{}，构建 → 签名 → 广播 → 监听）===", network.name());
    println!("发送方: {}", from_addr);
    println!("接收方: {}", to_addr);
    println!("金额(wei): {}", value_wei);

    println!("\n1. 构建原生 ETH 转账交易...");
    let tx_json = chain
        .evm_build_native_transfer(&provider, &from_addr, &to_addr, &value_wei, None)
        .await?;
    println!("   构建成功");

    println!("2. 使用 ETH_PRIVATE_KEY 签名...");
    let signed_hex = sign_ethereum_transaction(&tx_json, &private_key)?;
    println!("   签名成功");

    let chain_arc = Arc::new(chain);
    let sender = TransactionSender::new(chain_arc.clone());
    let monitor = TransactionMonitor::new(chain_arc);
    println!("3. 广播交易...");
    let tx_hash = match sender.send(&signed_hex).await {
        Ok(h) => h,
        Err(e) => {
            println!("❌ 广播失败: {}", e);
            let msg = e.to_string();
            if msg.contains("insufficient funds") || msg.contains("balance 0") {
                println!();
                println!("提示: 发送方地址需要原生 ETH 支付转账金额 + gas。");
                if network == EvmNetwork::ArbitrumSepolia {
                    println!("Arbitrum Sepolia 水龙头: https://faucet.quicknode.com/arbitrum/sepolia");
                } else {
                    println!("Sepolia 水龙头: https://sepoliafaucet.com 或 https://www.alchemy.com/faucets/ethereum-sepolia");
                }
            }
            return Ok(());
        }
    };
    println!("   交易哈希: {}", tx_hash);

    println!("4. 等待确认（超时 120s，轮询 3s）...");
    let status = monitor
        .wait_for_confirmation_with_timeout(&tx_hash, 120, Some(3000), Some(1))
        .await?;
    match status {
        TransactionStatus::Confirmed => println!("✅ 交易已确认!"),
        TransactionStatus::Failed => println!("❌ 交易失败!"),
        TransactionStatus::Pending => println!("⏳ 超时仍未确认"),
    }

    Ok(())
}

/// ERC20 查询与构建交易示例（不签名不广播）
///
/// 环境变量：
/// - ERC20_CONTRACT_ADDRESS：代币合约地址（0x 格式），未设置时使用 SDK EvmNetwork::Sepolia.usdt_contract()
/// - EVM_RPC_URL：可选，未设置时从 SDK EvmNetwork::Sepolia.urls() 依次尝试直到可用
pub async fn run_erc20_demo() -> Result<(), Box<dyn std::error::Error>> {
    let network = config::current_evm_network();
    let rpc_url = config::evm_rpc_url(network).await;
    let from_sdk = env::var("ERC20_CONTRACT_ADDRESS").is_err();
    let contract = env::var("ERC20_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| network.usdt_contract().to_string());

    let provider = EvmRpcProvider::new(rpc_url.clone());
    let chain = EvmChain::new(chains_sdk::types::ChainType::Ethereum, Arc::new(provider.clone()));

    println!("=== ERC20 代币功能示例（{}）===", network.name());
    println!("地址: {}", config::EVM_EXAMPLE_ADDR);
    if from_sdk {
        println!("合约: {}（来自 SDK {} 默认）", contract, network.name());
    } else {
        println!("合约: {}", contract);
    }
    println!("RPC:  {}", rpc_url);

    match chain.erc20_balance_of(&provider, config::EVM_EXAMPLE_ADDR, &contract).await {
        Ok(balance) => println!("余额: {}", balance),
        Err(e) => println!("查询余额失败: {}", e),
    }

    match chain.erc20_symbol(&provider, &contract).await {
        Ok(symbol) => println!("符号: {}", symbol),
        Err(e) => println!("查询符号失败: {}", e),
    }

    match chain.erc20_decimals(&provider, &contract).await {
        Ok(decimals) => println!("精度: {}", decimals),
        Err(e) => println!("查询精度失败: {}", e),
    }

    match chain.erc20_name(&provider, &contract).await {
        Ok(name) => println!("名称: {}", name),
        Err(e) => println!("查询名称失败: {}", e),
    }

    match chain.erc20_total_supply(&provider, &contract).await {
        Ok(supply) => println!("总供应量: {}", supply),
        Err(e) => println!("查询总供应失败: {}", e),
    }

    match chain
        .erc20_build_transfer(&provider, config::EVM_EXAMPLE_ADDR, config::EVM_EXAMPLE_ADDR, &contract, "0", None)
        .await
    {
        Ok(tx_json) => {
            println!("构建 ERC20 转账交易成功 (未签名未发送)");
            println!("交易 JSON 长度: {} 字节", tx_json.len());
        }
        Err(e) => println!("构建 ERC20 转账交易失败: {}", e),
    }

    Ok(())
}

/// 按 SDK 验证 ERC20 API（Sepolia 测试网）
pub async fn run_verify_erc20() -> Result<(), Box<dyn std::error::Error>> {
    let network = config::current_evm_network();
    let rpc_url = config::evm_rpc_url(network).await;
    let contract = env::var("ERC20_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| network.usdt_contract().to_string());

    let provider = EvmRpcProvider::new(rpc_url.clone());
    let chain = EvmChain::new(chains_sdk::types::ChainType::Ethereum, Arc::new(provider.clone()));

    println!("=== 根据 SDK 验证 ERC20 API（{}）===", network.name());
    println!("地址: {}", config::EVM_EXAMPLE_ADDR);
    println!("合约: {}", contract);
    println!();

    let mut ok = 0;
    let mut fail = 0;

    if chain.erc20_balance_of(&provider, config::EVM_EXAMPLE_ADDR, &contract).await.is_ok() {
        println!("✅ erc20_balance_of");
        ok += 1;
    } else {
        println!("❌ erc20_balance_of");
        fail += 1;
    }
    if chain.erc20_symbol(&provider, &contract).await.is_ok() {
        println!("✅ erc20_symbol");
        ok += 1;
    } else {
        println!("❌ erc20_symbol");
        fail += 1;
    }
    if chain.erc20_decimals(&provider, &contract).await.is_ok() {
        println!("✅ erc20_decimals");
        ok += 1;
    } else {
        println!("❌ erc20_decimals");
        fail += 1;
    }
    if chain.erc20_name(&provider, &contract).await.is_ok() {
        println!("✅ erc20_name");
        ok += 1;
    } else {
        println!("❌ erc20_name");
        fail += 1;
    }
    if chain.erc20_total_supply(&provider, &contract).await.is_ok() {
        println!("✅ erc20_total_supply");
        ok += 1;
    } else {
        println!("❌ erc20_total_supply");
        fail += 1;
    }
    if chain
        .erc20_allowance(&provider, config::EVM_EXAMPLE_ADDR, config::EVM_EXAMPLE_ADDR, &contract)
        .await
        .is_ok()
    {
        println!("✅ erc20_allowance");
        ok += 1;
    } else {
        println!("❌ erc20_allowance");
        fail += 1;
    }
    if chain
        .erc20_token_info(&provider, config::EVM_EXAMPLE_ADDR, &contract)
        .await
        .is_ok()
    {
        println!("✅ erc20_token_info");
        ok += 1;
    } else {
        println!("❌ erc20_token_info");
        fail += 1;
    }
    match chain
        .erc20_build_transfer(&provider, config::EVM_EXAMPLE_ADDR, config::EVM_EXAMPLE_ADDR, &contract, "0", None)
        .await
    {
        Ok(tx_json) => {
            if serde_json::from_str::<serde_json::Value>(&tx_json).is_ok() {
                println!("✅ erc20_build_transfer");
                ok += 1;
            } else {
                println!("❌ erc20_build_transfer (非合法 JSON)");
                fail += 1;
            }
        }
        Err(_) => {
            println!("❌ erc20_build_transfer");
            fail += 1;
        }
    }
    match chain
        .erc20_build_approve(&provider, config::EVM_EXAMPLE_ADDR, config::EVM_EXAMPLE_ADDR, &contract, "0", None)
        .await
    {
        Ok(tx_json) => {
            if serde_json::from_str::<serde_json::Value>(&tx_json).is_ok() {
                println!("✅ erc20_build_approve");
                ok += 1;
            } else {
                println!("❌ erc20_build_approve (非合法 JSON)");
                fail += 1;
            }
        }
        Err(_) => {
            println!("❌ erc20_build_approve");
            fail += 1;
        }
    }
    match chain
        .erc20_build_transfer_from(&provider, config::EVM_EXAMPLE_ADDR, config::EVM_EXAMPLE_ADDR, &contract, "0", None)
        .await
    {
        Ok(tx_json) => {
            if serde_json::from_str::<serde_json::Value>(&tx_json).is_ok() {
                println!("✅ erc20_build_transfer_from");
                ok += 1;
            } else {
                println!("❌ erc20_build_transfer_from (非合法 JSON)");
                fail += 1;
            }
        }
        Err(_) => {
            println!("❌ erc20_build_transfer_from");
            fail += 1;
        }
    }

    println!();
    println!("合计: {} 通过, {} 失败", ok, fail);
    if fail > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// 将“代币数量”（如 120 USDT）按精度换算为最小单位（raw）。
/// 例如 decimals=6 时 120 → 120_000_000
fn human_amount_to_raw(human: &str, decimals: u8) -> Result<String, Box<dyn std::error::Error>> {
    let human_f: f64 = human
        .trim()
        .parse()
        .map_err(|_| format!("ERC20_AMOUNT 应为数字，当前: {}", human))?;
    if human_f < 0.0 {
        return Err("ERC20_AMOUNT 不能为负数".into());
    }
    let raw_f = human_f * (10f64).powi(decimals as i32);
    let raw_u128 = raw_f.round() as u128;
    Ok(raw_u128.to_string())
}

/// 全自动 ERC20 流程：构建 → 签名 → 广播 → 监听
///
/// 环境变量：
/// - ETH_PRIVATE_KEY：发送方私钥，64 位十六进制
/// - ERC20_CONTRACT_ADDRESS：代币合约地址
/// - ETH_FROM_ADDRESS、ETH_TO_ADDRESS：可选
/// - ERC20_AMOUNT：代币数量（人类可读），如 120 表示 120 USDT（按合约精度换算）；默认 "0"
/// - EVM_RPC_URL：可选
pub async fn run_full_flow_erc20() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = match env::var("ETH_PRIVATE_KEY") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("未设置 ETH_PRIVATE_KEY。");
            eprintln!("示例: export ETH_PRIVATE_KEY=你的64位十六进制私钥");
            return Ok(());
        }
    };
    let network = config::current_evm_network();
    let rpc_url = config::evm_rpc_url(network).await;
    let contract = env::var("ERC20_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| network.usdt_contract().to_string());
    let from_addr = env::var("ETH_FROM_ADDRESS").unwrap_or_else(|_| config::EVM_EXAMPLE_ADDR.to_string());
    let to_addr = env::var("ETH_TO_ADDRESS").unwrap_or_else(|_| config::EVM_EXAMPLE_ADDR.to_string());
    let amount_human = env::var("ERC20_AMOUNT").unwrap_or_else(|_| "0".to_string());

    let provider = EvmRpcProvider::new(rpc_url.clone());
    let chain = EvmChain::new(chains_sdk::types::ChainType::Ethereum, Arc::new(provider.clone()));

    let decimals = chain.erc20_decimals(&provider, &contract).await?;
    let amount_raw = human_amount_to_raw(&amount_human, decimals)?;

    let key_address = ethereum_address_from_private_key(&private_key)
        .unwrap_or_else(|_| "?".to_string());
    let from_normalized = from_addr.trim_start_matches("0x").to_lowercase();
    let key_normalized = key_address.trim_start_matches("0x").to_lowercase();
    let key_matches_from = from_normalized == key_normalized;

    println!("=== 全自动 ERC20 流程（构建 → 签名 → 广播 → 监听）===");
    println!("发送方: {}", from_addr);
    println!("私钥对应地址: {} {}", key_address, if key_matches_from { "✓" } else { "⚠ 与发送方不一致" });
    println!("接收方: {}", to_addr);
    println!("合约:   {}", contract);
    println!("金额:   {}（= {} 最小单位，精度 {}）", amount_human, amount_raw, decimals);
    if !key_matches_from {
        println!();
        println!("⚠ 注意: ETH_PRIVATE_KEY 对应的地址是 {}，与发送方 {} 不一致。", key_address, from_addr);
        println!("  链上会从「私钥对应地址」扣 gas，若该地址余额为 0 会报 insufficient funds。");
        println!("  请使用与发送方地址对应的私钥，或把 ETH 转到私钥对应地址。");
        println!();
    }

    println!("\n1. 构建 ERC20 转账交易...");
    let tx_json = chain
        .erc20_build_transfer(&provider, &from_addr, &to_addr, &contract, &amount_raw, None)
        .await?;
    println!("   构建成功");

    println!("2. 使用 ETH_PRIVATE_KEY 签名...");
    let signed_hex = sign_ethereum_transaction(&tx_json, &private_key)?;
    println!("   签名成功");

    let chain_arc = Arc::new(chain);
    let sender = TransactionSender::new(chain_arc.clone());
    let monitor = TransactionMonitor::new(chain_arc);
    println!("3. 广播交易...");
    let tx_hash = match sender.send(&signed_hex).await {
        Ok(h) => h,
        Err(e) => {
            println!("❌ 广播失败: {}", e);
            let msg = e.to_string();
            if msg.contains("insufficient funds") || msg.contains("balance 0") {
                println!();
                println!("提示: 链上从「私钥对应地址」扣 gas，当前私钥对应: {}", key_address);
                if !key_matches_from {
                    println!("  您设置的发送方是 {}，与私钥对应地址不一致，请确认 ETH_PRIVATE_KEY 对应有 ETH 的地址。", from_addr);
                } else if network == EvmNetwork::ArbitrumSepolia {
                    println!("  请确认该地址在 Arbitrum Sepolia 上有少量原生 ETH：");
                    println!("  - https://faucet.quicknode.com/arbitrum/sepolia");
                } else {
                    println!("  请确认该地址在 Sepolia 上有少量原生 ETH：");
                    println!("  - https://sepoliafaucet.com");
                    println!("  - https://www.alchemy.com/faucets/ethereum-sepolia");
                }
            }
            return Ok(());
        }
    };
    println!("   交易哈希: {}", tx_hash);

    println!("4. 等待确认（超时 120s，轮询 3s）...");
    let status = monitor
        .wait_for_confirmation_with_timeout(&tx_hash, 120, Some(3000), Some(1))
        .await?;
    match status {
        TransactionStatus::Confirmed => println!("✅ 交易已确认!"),
        TransactionStatus::Failed => println!("❌ 交易失败!"),
        TransactionStatus::Pending => println!("⏳ 超时仍未确认"),
    }

    Ok(())
}
