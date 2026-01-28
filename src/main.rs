mod trc20;

use chains_sdk::balance::BalanceProvider;
use chains_sdk::chain::tron::TronChain;
use chains_sdk::transaction::{TransactionMonitor, TransactionSender, TransactionStatus};
use std::env;
use std::sync::Arc;

/// 示例地址（Tron Nile 测试网）
const ADDR: &str = "TG2D8vTp4xHBB2vhVbgHK2AhA2p9wY4q9M";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "balance".to_string());

    match cmd.as_str() {
        "balance" => run_balance().await?,
        "trc20" => trc20::run_trc20_demo().await?,
        "usdt-balance" => trc20::run_usdt_balance().await?,
        "verify-trc20" => trc20::run_verify_trc20().await?,
        "trx-transfer" => trc20::run_trx_transfer().await?,
        "monitor" => run_monitor_demo().await?,
        "send-demo" => run_send_demo().await?,
        "send-signed" => run_send_signed().await?,
        "full-flow" => trc20::run_full_flow().await?,
        _ => {
            eprintln!("未知命令: {}", cmd);
            eprintln!("用法: cargo run -- <balance|trc20|usdt-balance|verify-trc20|trx-transfer|monitor|send-demo|send-signed|full-flow>");
        }
    }

    Ok(())
}

/// 查询 TRX 余额（原先的示例）
async fn run_balance() -> Result<(), Box<dyn std::error::Error>> {
    let chain = Arc::new(TronChain::nile());
    let balance_provider = BalanceProvider::new(chain.clone());

    println!("=== TRX 余额查询示例 ===");
    println!("地址: {}", ADDR);

    let b = balance_provider.get_balance(&ADDR.to_string()).await?;
    println!("TRX 余额: {}", b.balance);

    Ok(())
}

/// 交易监听示例：根据给定 tx_hash 等待确认
async fn run_monitor_demo() -> Result<(), Box<dyn std::error::Error>> {
    let chain = Arc::new(TronChain::nile());
    let monitor = TransactionMonitor::new(chain);

    let tx_hash = match env::var("TX_HASH") {
        Ok(v) => v,
        Err(_) => {
            println!("请通过环境变量 TX_HASH 提供要监听的交易哈希。");
            println!("例如：export TX_HASH=your_tx_hash");
            return Ok(());
        }
    };

    println!("等待交易确认: {}", tx_hash);

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

/// 发送交易 + 等待确认示例
///
/// 注意：这里仍然用的是 SDK 示例里的假数据 raw_tx，真实使用时应替换为
/// 你自己“构建并完成签名”的原始交易 JSON。
async fn run_send_demo() -> Result<(), Box<dyn std::error::Error>> {
    let chain = Arc::new(TronChain::nile());
    let sender = TransactionSender::new(chain.clone());
    let monitor = TransactionMonitor::new(chain);

    // 这里直接复用 SDK 示例中的 raw_tx。真正发送上链前，务必替换为你自己签名后的交易。
    let raw_tx = r#"
    {
        "raw_data": {
            "contract": [{
                "parameter": {
                    "value": {
                        "owner_address": "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf",
                        "to_address": "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf",
                        "amount": 1000000
                    }
                },
                "type": "TransferContract"
            }],
            "ref_block_bytes": "0000",
            "ref_block_hash": "0000000000000000",
            "expiration": 0,
            "timestamp": 0
        },
        "signature": []
    }
    "#;

    println!("发送交易（示例数据，仅供调试接口用法）...");

    match sender.send(raw_tx).await {
        Ok(tx_hash) => {
            println!("交易已发送! 交易哈希: {}", tx_hash);

            println!("等待交易确认...");
            let status = monitor
                .wait_for_confirmation_with_timeout(&tx_hash, 60, Some(3000), Some(20))
                .await?;

            match status {
                TransactionStatus::Confirmed => println!("✅ 交易已确认!"),
                TransactionStatus::Failed => println!("❌ 交易失败!"),
                TransactionStatus::Pending => {
                    println!("⏳ 交易仍在等待确认（超时）");
                }
            }
        }
        Err(e) => {
            println!("❌ 发送交易失败: {}", e);
        }
    }

    Ok(())
}

/// 真实上链：发送你自己已经“构建并签名”的原始交易 JSON，并等待确认。
///
/// 为了避免在代码中处理私钥，这里约定通过环境变量或文件传入已签名交易：
/// - 优先读取环境变量 SIGNED_TX_JSON
/// - 否则如果设置了 SIGNED_TX_PATH，则从对应文件中读取 JSON
async fn run_send_signed() -> Result<(), Box<dyn std::error::Error>> {
    let chain = Arc::new(TronChain::nile());
    let sender = TransactionSender::new(chain.clone());
    let monitor = TransactionMonitor::new(chain);

    // 1. 获取已签名交易 JSON
    let raw_tx = if let Ok(json) = env::var("SIGNED_TX_JSON") {
        json
    } else if let Ok(path) = env::var("SIGNED_TX_PATH") {
        println!("从文件读取已签名交易: {}", path);
        std::fs::read_to_string(path)?
    } else {
        println!("未提供已签名交易。");
        println!("请通过以下两种方式之一提供：");
        println!("1）环境变量 SIGNED_TX_JSON 直接传入完整 JSON；");
        println!("2）环境变量 SIGNED_TX_PATH 指向包含 JSON 的文件路径。");
        return Ok(());
    };

    println!("发送已签名交易到 Tron Nile...");

    // 2. 发送交易
    match sender.send(&raw_tx).await {
        Ok(tx_hash) => {
            println!("✅ 交易已发送! 交易哈希: {}", tx_hash);

            // 3. 等待确认
            println!("等待交易确认...");
            let status = monitor
                .wait_for_confirmation_with_timeout(&tx_hash, 60, Some(3000), None)
                .await?;

            match status {
                TransactionStatus::Confirmed => println!("✅ 交易已确认!"),
                TransactionStatus::Failed => println!("❌ 交易失败!"),
                TransactionStatus::Pending => {
                    println!("⏳ 交易仍在等待确认（超时）");
                }
            }
        }
        Err(e) => {
            println!("❌ 发送交易失败: {}", e);
        }
    }

    Ok(())
}
