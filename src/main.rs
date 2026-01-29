mod config;
mod erc20;
mod trc20;

use chains_sdk::balance::BalanceProvider;
use chains_sdk::chain::tron::TronChain;
use chains_sdk::transaction::{TransactionMonitor, TransactionStatus};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "help".to_string());

    match cmd.as_str() {
        // Tron (Nile / Mainnet)
        "tron-balance" => run_tron_balance().await?,
        "tron-trc20" => trc20::run_trc20_demo().await?,
        "tron-usdt-balance" => trc20::run_usdt_balance().await?,
        "tron-verify-trc20" => trc20::run_verify_trc20().await?,
        "tron-transfer" => trc20::run_trx_transfer().await?,
        "tron-full-flow" => trc20::run_full_flow().await?,
        "tron-monitor" => run_tron_monitor().await?,
        // Ethereum 原生 (Sepolia)
        "eth-balance" => erc20::run_eth_balance().await?,
        "eth-transfer" => erc20::run_eth_transfer().await?,
        "eth-monitor" => erc20::run_eth_monitor().await?,
        // ERC20 (Sepolia)
        "erc20-demo" => erc20::run_erc20_demo().await?,
        "erc20-verify" => erc20::run_verify_erc20().await?,
        "erc20-full-flow" => erc20::run_full_flow_erc20().await?,
        "help" | "-h" | "--help" | _ => print_usage(cmd.as_str()),
    }

    Ok(())
}

fn print_usage(unknown: &str) {
    if unknown != "help" && unknown != "-h" && unknown != "--help" {
        eprintln!("未知命令: {}", unknown);
        eprintln!();
    }
    eprintln!("用法: cargo run -- <命令>");
    eprintln!();
    eprintln!("网络选择：");
    eprintln!("  TRON_NETWORK   Tron: nile（默认）| mainnet | shasta");
    eprintln!("  EVM_NETWORK    EVM:  sepolia（默认）| arbitrum-sepolia | arbitrum-one | mainnet");
    eprintln!();
    eprintln!("Tron:");
    eprintln!("  tron-balance          TRX 余额");
    eprintln!("  tron-trc20            TRC20 代币信息与构建转账");
    eprintln!("  tron-usdt-balance     USDT 余额（按 TRON_NETWORK）");
    eprintln!("  tron-verify-trc20     验证 TRC20 API");
    eprintln!("  tron-transfer         TRX 原生转账");
    eprintln!("  tron-full-flow        TRC20 全流程（构建→签名→广播→监听）");
    eprintln!("  tron-monitor          监听 Tron 交易（TX_HASH）");
    eprintln!();
    eprintln!("EVM 原生 ETH:");
    eprintln!("  eth-balance           原生 ETH 余额");
    eprintln!("  eth-transfer          原生 ETH 转账全流程");
    eprintln!("  eth-monitor           监听交易（TX_HASH，含 ETH/ERC20）");
    eprintln!();
    eprintln!("ERC20:");
    eprintln!("  erc20-demo            ERC20 代币信息与构建转账");
    eprintln!("  erc20-verify          验证 ERC20 API");
    eprintln!("  erc20-full-flow       ERC20 全流程（构建→签名→广播→监听）");
    eprintln!();
    eprintln!("  help                  显示此帮助");
}

/// 查询 TRX 余额（网络由 TRON_NETWORK 指定，默认 nile）
async fn run_tron_balance() -> Result<(), Box<dyn std::error::Error>> {
    let network = trc20::current_tron_network();
    let chain = Arc::new(TronChain::from_network(network));
    let balance_provider = BalanceProvider::new(chain.clone());

    println!("=== TRX 余额查询（{}）===", network.name());
    println!("地址: {}", config::TRON_EXAMPLE_ADDR);

    let b = balance_provider.get_balance(&config::TRON_EXAMPLE_ADDR.to_string()).await?;
    println!("TRX 余额: {}", b.balance);

    Ok(())
}

/// 监听 Tron 交易（TX_HASH，网络由 TRON_NETWORK 指定）
async fn run_tron_monitor() -> Result<(), Box<dyn std::error::Error>> {
    let tx_hash = match env::var("TX_HASH") {
        Ok(v) => v,
        Err(_) => {
            println!("请通过环境变量 TX_HASH 提供要监听的 Tron 交易哈希。");
            println!("例如：export TX_HASH=your_tx_hash");
            return Ok(());
        }
    };

    let network = trc20::current_tron_network();
    let chain = Arc::new(TronChain::from_network(network));
    let monitor = TransactionMonitor::new(chain);

    println!("=== 监听 Tron 交易（{}）===", network.name());
    println!("交易哈希: {}", tx_hash);

    let status = monitor
        .wait_for_confirmation(&tx_hash, Some(10), Some(3000), Some(20))
        .await?;

    match status {
        TransactionStatus::Confirmed => println!("✅ 交易已确认!"),
        TransactionStatus::Failed => println!("❌ 交易失败!"),
        TransactionStatus::Pending => println!("⏳ 交易仍在等待确认（达到最大尝试次数）"),
    }

    Ok(())
}
