use chains_sdk::balance::BalanceProvider;
use chains_sdk::chain::tron::TronChain;
use chains_sdk::rpc::chains::tron::{TronNetwork, TronRpcProvider};
use chains_sdk::transaction::{TransactionMonitor, TransactionSender, TransactionStatus};
use secp256k1::Secp256k1;
use std::env;
use std::sync::Arc;

/// 示例地址（Tron Nile 测试网）
const ADDR: &str = "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "balance".to_string());

    match cmd.as_str() {
        "balance" => run_balance().await?,
        "trc20" => run_trc20_demo().await?,
        "monitor" => run_monitor_demo().await?,
        "send-demo" => run_send_demo().await?,
        "send-signed" => run_send_signed().await?,
        "full-flow" => run_full_flow().await?,
        _ => {
            eprintln!("未知命令: {}", cmd);
            eprintln!("用法: cargo run -- <balance|trc20|monitor|send-demo|send-signed|full-flow>");
        }
    }

    Ok(())
}

/// 使用私钥对 Tron 交易 JSON 签名（对 txID 做 ECDSA secp256k1 签名，65 字节 r+s+v 转 hex 写入 signature）
fn sign_tron_transaction(tx_json: &str, private_key_hex: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut tx: serde_json::Value = serde_json::from_str(tx_json)?;
    let txid_hex = tx
        .get("txID")
        .and_then(|v| v.as_str())
        .ok_or("transaction JSON 缺少 txID")?;
    if txid_hex.len() != 64 {
        return Err("txID 应为 64 位十六进制".into());
    }
    let txid_bytes: [u8; 32] = hex::decode(txid_hex)
        .map_err(|e| format!("txID hex 解码失败: {}", e))?
        .try_into()
        .map_err(|_| "txID 长度非 32 字节")?;

    let pk_hex = private_key_hex
        .trim()
        .trim_start_matches("0x")
        .trim();
    if pk_hex.len() != 64 {
        return Err(format!(
            "TRON_PRIVATE_KEY 应为 32 字节私钥的十六进制（64 个 0-9/a-f 字符），可带 0x 前缀；当前长度 {}。若你使用的是助记词，请先用 TronLink/钱包或命令行工具从助记词导出私钥 hex 再填入。",
            pk_hex.len()
        ).into());
    }
    let pk_bytes = hex::decode(pk_hex).map_err(|e| format!("私钥 hex 解码失败: {}", e))?;
    let secret_key = secp256k1::SecretKey::from_slice(&pk_bytes)?;

    let secp = Secp256k1::new();
    let message = secp256k1::Message::from_digest(txid_bytes);
    let sig = secp.sign_ecdsa_recoverable(&message, &secret_key);
    let (recovery_id, sig_compact) = sig.serialize_compact();
    let mut sig_65 = [0u8; 65];
    sig_65[..64].copy_from_slice(&sig_compact);
    sig_65[64] = recovery_id.to_i32() as u8;
    let sig_hex = hex::encode(sig_65);

    tx["signature"] = serde_json::json!([sig_hex]);
    Ok(serde_json::to_string(&tx)?)
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

/// TRC20 查询与构建交易示例（不签名不广播，安全演示）
async fn run_trc20_demo() -> Result<(), Box<dyn std::error::Error>> {
    let chain = TronChain::nile();
    let provider = TronRpcProvider::from_network(TronNetwork::Nile);

    println!("=== TRC20 代币功能示例 ===");
    println!("地址: {}", ADDR);

    let contract = match env::var("TRC20_CONTRACT_ADDRESS") {
        Ok(v) => v,
        Err(_) => {
            println!("未设置环境变量 TRC20_CONTRACT_ADDRESS，示例只演示用法。");
            println!("例如：export TRC20_CONTRACT_ADDRESS=<代币合约地址>");
            return Ok(());
        }
    };

    // 余额
    match chain.trc20_balance_of(&provider, ADDR, &contract).await {
        Ok(balance) => println!("余额: {}", balance),
        Err(e) => println!("查询余额失败: {}", e),
    }

    // 符号
    match chain.trc20_symbol(&provider, &contract).await {
        Ok(symbol) => println!("符号: {}", symbol),
        Err(e) => println!("查询符号失败: {}", e),
    }

    // 精度
    match chain.trc20_decimals(&provider, &contract).await {
        Ok(decimals) => println!("精度: {}", decimals),
        Err(e) => println!("查询精度失败: {}", e),
    }

    // 名称
    match chain.trc20_name(&provider, &contract).await {
        Ok(name) => println!("名称: {}", name),
        Err(e) => println!("查询名称失败: {}", e),
    }

    // 总供应
    match chain.trc20_total_supply(&provider, &contract).await {
        Ok(supply) => println!("总供应量: {}", supply),
        Err(e) => println!("查询总供应失败: {}", e),
    }

    // 构建一个示例 TRC20 转账交易（不签名不发送）
    match chain
        .trc20_build_transfer(
            &provider,
            ADDR,
            ADDR,
            &contract,
            "1000000",
            Some(100_000_000),
        )
        .await
    {
        Ok(tx_json) => {
            println!("构建 TRC20 转账交易成功 (未签名未发送)");
            println!("交易 JSON 长度: {} 字节", tx_json.len());
        }
        Err(e) => println!("构建 TRC20 转账交易失败: {}", e),
    }

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

/// 方案 B：全自动流程 —— 构建 TRC20 转账 → 私钥签名 → 广播 → 监听确认/失败
///
/// 环境变量：
/// - TRON_PRIVATE_KEY（必填）：发送方私钥，64 位十六进制（32 字节）
/// - TRC20_CONTRACT_ADDRESS（必填）：TRC20 合约地址
/// - TRON_FROM_ADDRESS（可选）：发送方地址，默认示例地址
/// - TRON_TO_ADDRESS（可选）：接收方地址，默认同 FROM
/// - TRC20_AMOUNT（可选）：转账金额（最小单位），默认 "1000000"
/// - TRC20_FEE_LIMIT（可选）：费用上限 sun，默认 100_000_000
async fn run_full_flow() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = match env::var("TRON_PRIVATE_KEY") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("未设置 TRON_PRIVATE_KEY。");
            eprintln!("示例: export TRON_PRIVATE_KEY=你的64位十六进制私钥");
            return Ok(());
        }
    };
    let contract = match env::var("TRC20_CONTRACT_ADDRESS") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("未设置 TRC20_CONTRACT_ADDRESS。");
            return Ok(());
        }
    };
    let from_addr = env::var("TRON_FROM_ADDRESS").unwrap_or_else(|_| ADDR.to_string());
    let to_addr = env::var("TRON_TO_ADDRESS").unwrap_or_else(|_| ADDR.to_string());
    let amount = env::var("TRC20_AMOUNT").unwrap_or_else(|_| "1000000".to_string());
    let fee_limit: i64 = env::var("TRC20_FEE_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000_000);

    let chain = TronChain::nile();
    let provider = TronRpcProvider::from_network(TronNetwork::Nile);

    println!("=== 全自动 TRC20 流程（构建 → 签名 → 广播 → 监听）===");
    println!("发送方: {}", from_addr);
    println!("接收方: {}", to_addr);
    println!("合约: {}", contract);
    println!("金额(最小单位): {}", amount);

    // 1. 构建交易
    println!("\n1. 构建 TRC20 转账交易...");
    let tx_json = chain
        .trc20_build_transfer(&provider, &from_addr, &to_addr, &contract, &amount, Some(fee_limit))
        .await?;
    println!("   构建成功");

    // 2. 签名
    println!("2. 使用 TRON_PRIVATE_KEY 签名...");
    let signed_tx = sign_tron_transaction(&tx_json, &private_key)?;
    println!("   签名成功");

    // 3. 广播（需要 Arc<dyn Blockchain>）
    let chain_arc = Arc::new(chain);
    let sender = TransactionSender::new(chain_arc.clone());
    let monitor = TransactionMonitor::new(chain_arc);
    println!("3. 广播交易...");
    let tx_hash = match sender.send(&signed_tx).await {
        Ok(h) => h,
        Err(e) => {
            println!("❌ 广播失败: {}", e);
            return Ok(());
        }
    };
    println!("   交易哈希: {}", tx_hash);

    // 4. 等待最终确认（Tron 常用 19 个区块后视为不可逆）
    const MIN_CONFIRMATIONS: u32 = 19;
    println!("4. 等待最终确认（至少 {} 个区块，超时 120s，轮询 3s）...", MIN_CONFIRMATIONS);
    let status = monitor
        .wait_for_confirmation_with_timeout(&tx_hash, 120, Some(3000), Some(MIN_CONFIRMATIONS))
        .await?;
    match status {
        TransactionStatus::Confirmed => {
            println!("✅ 交易已最终确认（{} 个区块确认）!", MIN_CONFIRMATIONS);
        }
        TransactionStatus::Failed => println!("❌ 交易失败!"),
        TransactionStatus::Pending => println!("⏳ 超时仍未达到 {} 个区块确认", MIN_CONFIRMATIONS),
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
