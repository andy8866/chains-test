//! TRC20 代币测试与示例（只读查询、构建交易、全自动流程）

use chains_sdk::chain::tron::{sign_tron_transaction, TronChain};
use chains_sdk::rpc::chains::tron::{TronNetwork, TronRpcProvider};
use chains_sdk::transaction::{TransactionMonitor, TransactionSender, TransactionStatus};
use std::env;
use std::sync::Arc;

/// 示例地址（Tron Nile 测试网）
const ADDR: &str = "TG2D8vTp4xHBB2vhVbgHK2AhA2p9wY4q9M";

/// 查询 TRC20 USDT 余额（主网）
///
/// 环境变量：TRON_ADDRESS（必填）— 要查询的 Tron 地址
pub async fn run_usdt_balance() -> Result<(), Box<dyn std::error::Error>> {
    let address = match env::var("TRON_ADDRESS") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("未设置 TRON_ADDRESS。");
            eprintln!("示例: export TRON_ADDRESS=你的Tron地址");
            return Ok(());
        }
    };

    let chain = TronChain::mainnet();
    let provider = TronRpcProvider::from_network(TronNetwork::Mainnet);
    let usdt_contract = TronNetwork::Mainnet.usdt_contract();

    println!("=== TRC20 USDT 余额（主网）===");
    println!("地址: {}", address);
    println!("合约: {}", usdt_contract);

    match chain.trc20_balance_of(&provider, &address, usdt_contract).await {
        Ok(raw) => {
            // USDT 精度为 6，原始值 / 1_000_000 = 显示金额
            let decimals = 1_000_000u64;
            let human = raw.parse::<u64>().unwrap_or(0) as f64 / decimals as f64;
            println!("余额(原始): {}", raw);
            println!("余额(USDT): {}", human);
        }
        Err(e) => {
            eprintln!("查询失败: {}", e);
        }
    }

    Ok(())
}

/// 根据 SDK 验证 TRC20 API：只读接口 + 构建交易（使用 Nile 测试网与 SDK 内置 USDT 合约）
pub async fn run_verify_trc20() -> Result<(), Box<dyn std::error::Error>> {
    let chain = TronChain::nile();
    let provider = TronRpcProvider::from_network(TronNetwork::Nile);
    let contract = TronNetwork::Nile.usdt_contract();

    println!("=== 根据 SDK 验证 TRC20 API（Nile 测试网）===");
    println!("地址: {}", ADDR);
    println!("合约: {}", contract);
    println!();

    let mut ok = 0;
    let mut fail = 0;

    // trc20_balance_of
    match chain.trc20_balance_of(&provider, ADDR, contract).await {
        Ok(b) => {
            println!("✅ trc20_balance_of  -> {}", b);
            ok += 1;
        }
        Err(e) => {
            println!("❌ trc20_balance_of  -> {}", e);
            fail += 1;
        }
    }

    // trc20_symbol
    match chain.trc20_symbol(&provider, contract).await {
        Ok(s) => {
            println!("✅ trc20_symbol       -> {}", s);
            ok += 1;
        }
        Err(e) => {
            println!("❌ trc20_symbol       -> {}", e);
            fail += 1;
        }
    }

    // trc20_decimals
    match chain.trc20_decimals(&provider, contract).await {
        Ok(d) => {
            println!("✅ trc20_decimals     -> {}", d);
            ok += 1;
        }
        Err(e) => {
            println!("❌ trc20_decimals     -> {}", e);
            fail += 1;
        }
    }

    // trc20_name
    match chain.trc20_name(&provider, contract).await {
        Ok(n) => {
            println!("✅ trc20_name         -> {}", n);
            ok += 1;
        }
        Err(e) => {
            println!("❌ trc20_name         -> {}", e);
            fail += 1;
        }
    }

    // trc20_total_supply
    match chain.trc20_total_supply(&provider, contract).await {
        Ok(t) => {
            println!("✅ trc20_total_supply -> {}", t);
            ok += 1;
        }
        Err(e) => {
            println!("❌ trc20_total_supply -> {}", e);
            fail += 1;
        }
    }

    // trc20_allowance(owner, spender, contract)
    match chain.trc20_allowance(&provider, ADDR, ADDR, contract).await {
        Ok(a) => {
            println!("✅ trc20_allowance    -> {}", a);
            ok += 1;
        }
        Err(e) => {
            println!("❌ trc20_allowance    -> {}", e);
            fail += 1;
        }
    }

    // trc20_token_info(owner, contract) -> Balance
    match chain.trc20_token_info(&provider, ADDR, contract).await {
        Ok(info) => {
            println!(
                "✅ trc20_token_info   -> balance={} symbol={:?} decimals={:?}",
                info.balance,
                info.symbol,
                info.decimals
            );
            ok += 1;
        }
        Err(e) => {
            println!("❌ trc20_token_info   -> {}", e);
            fail += 1;
        }
    }

    // trc20_build_transfer：校验返回 JSON 含 txID、raw_data
    match chain
        .trc20_build_transfer(&provider, ADDR, ADDR, contract, "1000000", Some(100_000_000))
        .await
    {
        Ok(tx_json) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&tx_json) {
                let has_tx_id = v.get("txID").and_then(|x| x.as_str()).is_some();
                let has_raw = v.get("raw_data").is_some();
                if has_tx_id && has_raw {
                    println!("✅ trc20_build_transfer -> txID + raw_data 存在");
                    ok += 1;
                } else {
                    println!("❌ trc20_build_transfer -> JSON 缺少 txID 或 raw_data");
                    fail += 1;
                }
            } else {
                println!("❌ trc20_build_transfer -> 非合法 JSON");
                fail += 1;
            }
        }
        Err(e) => {
            println!("❌ trc20_build_transfer -> {}", e);
            fail += 1;
        }
    }

    // trc20_build_approve：仅校验返回为合法 JSON 且含 txID
    match chain
        .trc20_build_approve(&provider, ADDR, ADDR, contract, "0", Some(100_000_000))
        .await
    {
        Ok(tx_json) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&tx_json) {
                if v.get("txID").is_some() {
                    println!("✅ trc20_build_approve -> txID 存在");
                    ok += 1;
                } else {
                    println!("❌ trc20_build_approve -> JSON 缺少 txID");
                    fail += 1;
                }
            } else {
                println!("❌ trc20_build_approve -> 非合法 JSON");
                fail += 1;
            }
        }
        Err(e) => {
            println!("❌ trc20_build_approve -> {}", e);
            fail += 1;
        }
    }

    // trc20_build_transfer_from：仅校验返回为合法 JSON 且含 txID
    match chain
        .trc20_build_transfer_from(&provider, ADDR, ADDR, contract, "0", Some(100_000_000))
        .await
    {
        Ok(tx_json) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&tx_json) {
                if v.get("txID").is_some() {
                    println!("✅ trc20_build_transfer_from -> txID 存在");
                    ok += 1;
                } else {
                    println!("❌ trc20_build_transfer_from -> JSON 缺少 txID");
                    fail += 1;
                }
            } else {
                println!("❌ trc20_build_transfer_from -> 非合法 JSON");
                fail += 1;
            }
        }
        Err(e) => {
            println!("❌ trc20_build_transfer_from -> {}", e);
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

/// TRC20 查询与构建交易示例（不签名不广播，安全演示）
pub async fn run_trc20_demo() -> Result<(), Box<dyn std::error::Error>> {
    let chain = TronChain::nile();
    let provider = TronRpcProvider::from_network(TronNetwork::Nile);

    println!("=== TRC20 代币功能示例 ===");
    println!("地址: {}", ADDR);

    // 优先环境变量，未设置则从 SDK 读取当前网络（Nile）的 USDT 合约地址
    let contract = env::var("TRC20_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| TronNetwork::Nile.usdt_contract().to_string());
    if env::var("TRC20_CONTRACT_ADDRESS").is_err() {
        println!("合约: {}（来自 SDK TronNetwork::Nile.usdt_contract()）", contract);
    } else {
        println!("合约: {}", contract);
    }

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

/// TRX 原生转账全流程：构建 → 签名 → 广播 → 监听确认/失败
///
/// 环境变量：
/// - TRON_PRIVATE_KEY（必填）：发送方私钥，64 位十六进制（32 字节）
/// - TRON_FROM_ADDRESS（可选）：发送方地址，默认示例地址
/// - TRON_TO_ADDRESS（可选）：接收方地址，默认同 FROM
/// - TRX_AMOUNT_SUN（可选）：转账金额 sun，默认 1000（0.001 TRX）
pub async fn run_trx_transfer() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = match env::var("TRON_PRIVATE_KEY") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("未设置 TRON_PRIVATE_KEY。");
            eprintln!("示例: export TRON_PRIVATE_KEY=你的64位十六进制私钥");
            return Ok(());
        }
    };
    let from_addr = env::var("TRON_FROM_ADDRESS").unwrap_or_else(|_| ADDR.to_string());
    let to_addr = env::var("TRON_TO_ADDRESS").unwrap_or_else(|_| ADDR.to_string());
    let amount_sun: i64 = env::var("TRX_AMOUNT_SUN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    let chain = TronChain::nile();
    let provider = TronRpcProvider::from_network(TronNetwork::Nile);

    println!("=== TRX 转账流程（构建 → 签名 → 广播 → 监听）===");
    println!("发送方: {}", from_addr);
    println!("接收方: {}", to_addr);
    println!("金额: {} sun ({:.6} TRX)", amount_sun, amount_sun as f64 / 1_000_000.0);

    // 1. 构建交易
    println!("\n1. 构建 TRX 转账交易...");
    let tx_json = chain
        .trx_build_transfer(&provider, &from_addr, &to_addr, amount_sun)
        .await?;

    println!("   构建成功");

    // 2. 签名
    println!("2. 使用 TRON_PRIVATE_KEY 签名...");
    let signed_tx = sign_tron_transaction(&tx_json, &private_key)?;
    println!("   签名成功");

    // 3. 广播
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

    // 4. 等待确认
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

/// 方案 B：全自动流程 —— 构建 TRC20 转账 → 私钥签名 → 广播 → 监听确认/失败
///
/// 环境变量：
/// - TRON_PRIVATE_KEY（必填）：发送方私钥，64 位十六进制（32 字节）
/// - TRC20_CONTRACT_ADDRESS（可选）：TRC20 合约地址，未设置则从 SDK 读取 Nile USDT 合约
/// - TRON_FROM_ADDRESS（可选）：发送方地址，默认示例地址
/// - TRON_TO_ADDRESS（可选）：接收方地址，默认同 FROM
/// - TRC20_AMOUNT（可选）：转账金额（最小单位），默认 "1000000"
/// - TRC20_FEE_LIMIT（可选）：费用上限 sun，默认 100_000_000
pub async fn run_full_flow() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = match env::var("TRON_PRIVATE_KEY") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("未设置 TRON_PRIVATE_KEY。");
            eprintln!("示例: export TRON_PRIVATE_KEY=你的64位十六进制私钥");
            return Ok(());
        }
    };
    // 优先环境变量，未设置则从 SDK 读取当前网络（Nile）的 USDT 合约地址
    let contract = env::var("TRC20_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| TronNetwork::Nile.usdt_contract().to_string());
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
    if env::var("TRC20_CONTRACT_ADDRESS").is_err() {
        println!("合约: {}（来自 SDK TronNetwork::Nile.usdt_contract()）", contract);
    } else {
        println!("合约: {}", contract);
    }
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
