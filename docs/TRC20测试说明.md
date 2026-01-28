# chains-test 测试说明

本文档说明如何运行本项目中的各项测试命令、所需环境变量及预期结果。  
项目基于 **Tron Nile 测试网**（部分命令使用主网），通过实际 RPC 调用验证 `chains-sdk` 行为。

---

## 前置条件

- 已安装 Rust / Cargo（建议 `rustup`）
- 同级目录存在 `chains`（chains-sdk）项目，且 `Cargo.toml` 中 `chains-sdk = { path = "../chains" }` 可解析
- 网络可访问 Tron 公共 RPC（如 `nile.trongrid.io`、`api.trongrid.io`）

---

## 命令一览

| 命令 | 说明 | 网络 |
|------|------|------|
| `balance` | 查询 TRX 余额 | Nile |
| `trc20` | TRC20 代币信息 + 构建转账（不签名不广播） | Nile |
| `usdt-balance` | 查询主网 TRC20 USDT 余额 | Mainnet |
| `verify-trc20` | 按 SDK 验证全部 TRC20 API | Nile |
| `trx-transfer` | TRX 原生转账：构建→签名→广播→监听 | Nile |
| `monitor` | 按交易哈希监听确认状态 | Nile |
| `send-demo` | 发送示例假交易（仅接口演示） | Nile |
| `send-signed` | 发送已签名交易并等待确认 | Nile |
| `full-flow` | 全自动 TRC20：构建→签名→广播→监听 | Nile |

---

## 1. balance — TRX 余额查询

**命令：**
```bash
cargo run -- balance
```

**环境变量：** 无（使用代码内示例地址）。

**预期：** 控制台输出 Nile 测试网示例地址的 TRX 余额，无报错即通过。

**验证点：** `BalanceProvider::get_balance`、Tron Nile RPC 连通性。

---

## 2. trc20 — TRC20 代币信息与构建转账

**命令：**
```bash
export TRC20_CONTRACT_ADDRESS=TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf
cargo run -- trc20
```

**环境变量：**

| 变量 | 必填 | 说明 |
|------|------|------|
| `TRC20_CONTRACT_ADDRESS` | 否 | Nile 上的 TRC20 合约地址；未设置时从 SDK 读取 `TronNetwork::Nile.usdt_contract()`（Nile USDT） |

**预期：** 依次输出该合约的余额、符号、精度、名称、总供应量，以及“构建 TRC20 转账交易成功”的提示。未设置合约时控制台会注明“来自 SDK TronNetwork::Nile.usdt_contract()”。

**验证点：** `trc20_balance_of`、`trc20_symbol`、`trc20_decimals`、`trc20_name`、`trc20_total_supply`、`trc20_build_transfer`。

---

## 3. usdt-balance — 主网 TRC20 USDT 余额

**命令：**
```bash
export TRON_ADDRESS=TG2D8vTp4xHBB2vhVbgHK2AhA2p9wY4q9M
cargo run -- usdt-balance
```

**环境变量：**

| 变量 | 必填 | 说明 |
|------|------|------|
| `TRON_ADDRESS` | 是 | 要查 USDT 余额的 Tron 地址（Base58） |

**预期：** 输出主网 USDT 合约地址、该地址的余额（原始值及换算后的 USDT 数量，精度 6）。

**验证点：** 主网 RPC、`TronNetwork::Mainnet.usdt_contract()`、`trc20_balance_of`。

---

## 4. verify-trc20 — 按 SDK 验证 TRC20 API

**命令：**
```bash
cargo run -- verify-trc20
```

**环境变量：** 无（使用 Nile 测试网及 SDK 内置 Nile USDT 合约地址）。

**预期：** 逐项输出 10 个 SDK TRC20 接口的验证结果（✅/❌），最后一行为“合计: 10 通过, 0 失败”。若有失败则进程退出码为 1。

**验证的 API：**

- `trc20_balance_of`
- `trc20_symbol`
- `trc20_decimals`
- `trc20_name`
- `trc20_total_supply`
- `trc20_allowance`
- `trc20_token_info`
- `trc20_build_transfer`（校验返回 JSON 含 `txID`、`raw_data`）
- `trc20_build_approve`（校验返回 JSON 含 `txID`）
- `trc20_build_transfer_from`（校验返回 JSON 含 `txID`）

**验证点：** SDK 提供的 TRC20 只读与构建交易接口在 Nile 上的端到端行为。

---

## 5. trx-transfer — TRX 原生转账（构建→签名→广播→监听）

**命令：**
```bash
export TRON_PRIVATE_KEY=79a5d62ebbe36b4e54fa5d795de9a2d4c528508a48e004cafbe0660a8d286e08
export TRON_FROM_ADDRESS=TG2D8vTp4xHBB2vhVbgHK2AhA2p9wY4q9M
export TRON_TO_ADDRESS=TPsXm9mBMn8WGoDQcvroGPQbb3WpP7K15t
cargo run -- trx-transfer
```

**环境变量：**

| 变量 | 必填 | 说明 |
|------|------|------|
| `TRON_PRIVATE_KEY` | 是 | 发送方私钥，64 位十六进制（32 字节），可带 `0x` 前缀 |
| `TRON_FROM_ADDRESS` | 否 | 发送方地址，默认示例地址 |
| `TRON_TO_ADDRESS` | 否 | 接收方地址，默认同 FROM |
| `TRX_AMOUNT_SUN` | 否 | 转账金额（sun），默认 1000（0.001 TRX） |

**预期：** 依次完成“构建 TRX 转账交易 → 本地签名 → 广播 → 等待确认”，并输出交易哈希与最终状态。

**验证点：** SDK `trx_build_transfer`、`sign_tron_transaction`、广播与监听的完整 TRX 转账链路。

---

## 6. monitor — 交易确认监听

**命令：**
```bash
export TX_HASH=<交易哈希>
cargo run -- monitor
```

**环境变量：**

| 变量 | 必填 | 说明 |
|------|------|------|
| `TX_HASH` | 是 | 要监听的交易哈希（Nile 上存在的交易） |

**预期：** 轮询后输出“交易已确认”或“交易失败”或“仍在等待确认”。

**验证点：** `TransactionMonitor::wait_for_confirmation`、交易状态解析。

---

## 7. send-demo — 示例发送（假数据）

**命令：**
```bash
cargo run -- send-demo
```

**环境变量：** 无。

**说明：** 使用内置假 `raw_tx` 调用发送与监听接口，不保证链上成功。

**预期：** 程序不 panic；可能因交易无效而得到发送失败或失败状态，属预期。

**验证点：** `TransactionSender::send`、`TransactionMonitor::wait_for_confirmation_with_timeout` 的调用流程。

---

## 8. send-signed — 发送已签名交易

**命令（二选一）：**

```bash
# 方式一：环境变量直接传 JSON
export SIGNED_TX_JSON='{"raw_data":{...},"signature":[...]}'
cargo run -- send-signed
```

```bash
# 方式二：从文件读取（推荐）
export SIGNED_TX_PATH=/path/to/signed_tx.json
cargo run -- send-signed
```

**环境变量：**

| 变量 | 必填 | 说明 |
|------|------|------|
| `SIGNED_TX_JSON` | 与 PATH 二选一 | 完整已签名交易 JSON 字符串 |
| `SIGNED_TX_PATH` | 与 JSON 二选一 | 包含上述 JSON 的文件路径 |

**预期：** 广播到 Nile，输出交易哈希，并轮询直至确认/失败/超时。

**验证点：** 真实已签名交易从发送到状态确认的完整流程。

---

## 9. full-flow — 全自动 TRC20 流程

**命令：**
```bash
export TRON_PRIVATE_KEY=<64 位十六进制私钥>
export TRC20_CONTRACT_ADDRESS=<Nile 上的 TRC20 合约地址>
# 可选：
# export TRON_FROM_ADDRESS=...
# export TRON_TO_ADDRESS=...
# export TRC20_AMOUNT=1000000
# export TRC20_FEE_LIMIT=100000000
cargo run -- full-flow
```

**环境变量：**

| 变量 | 必填 | 说明 |
|------|------|------|
| `TRON_PRIVATE_KEY` | 是 | 发送方私钥，64 位十六进制（32 字节），可带 `0x` 前缀；不可填助记词 |
| `TRC20_CONTRACT_ADDRESS` | 否 | Nile 上的 TRC20 合约；未设置时从 SDK 读取 `TronNetwork::Nile.usdt_contract()`（Nile USDT） |
| `TRON_FROM_ADDRESS` | 否 | 发送方地址，默认示例地址 |
| `TRON_TO_ADDRESS` | 否 | 接收方地址，默认同 FROM |
| `TRC20_AMOUNT` | 否 | 转账金额最小单位，默认 `1000000` |
| `TRC20_FEE_LIMIT` | 否 | 费用上限（sun），默认 `100000000` |

**预期：** 依次完成“构建 TRC20 转账 → 本地签名 → 广播 → 等待确认”，并输出交易哈希与最终状态。

**验证点：** `trc20_build_transfer`、本地 ECDSA 签名、`TransactionSender::send`、`TransactionMonitor::wait_for_confirmation_with_timeout` 整条链路。

**安全提示：** 私钥仅用于本地签名，不会上传；建议仅在测试网使用，勿泄露私钥。

---

## 推荐测试顺序

1. **连通性与只读：** `balance` → `verify-trc20` → `trc20`（需设置 `TRC20_CONTRACT_ADDRESS`）
2. **主网只读（可选）：** `usdt-balance`（需设置 `TRON_ADDRESS`）
3. **TRX 转账：** `trx-transfer`（需设置 `TRON_PRIVATE_KEY`）
4. **监听与发送：** `monitor`（需已有交易哈希）→ `send-demo` → `send-signed` 或 `full-flow`（需私钥与合约地址）

---

## 常见问题

- **未设置 TRC20_CONTRACT_ADDRESS：** 运行 `trc20` 时会提示设置，不设则只打印用法。
- **triggerSmartContract missing transaction：** 多为合约地址不属于当前网络（如把主网 USDT 地址用在 Nile），请改用当前网络上的合约地址。
- **TRON_PRIVATE_KEY 格式：** 必须为 32 字节私钥的 64 位十六进制；若只有助记词，需先用钱包或工具导出私钥 hex 再填入。
- **verify-trc20 失败：** 检查网络是否可访问 Nile RPC；若部分项失败，可根据输出中的错误信息排查 SDK 或网络问题。

---

## 相关文档

- **README.md** — 项目介绍与快速运行说明  
- **测试覆盖.md** — 测试覆盖范围与验证点说明（与本文档互补）
