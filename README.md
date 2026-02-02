# chains-test

用于**本地测试 `chains-sdk` 的示例 Rust 项目**，覆盖 Tron（TRX/TRC20）与 EVM（ETH/ERC20）的只读查询、构建交易、签名广播与交易监听。

## 前置条件

- 同级目录存在 `chains-sdk` 项目，且 `Cargo.toml` 中 `chains-sdk = { path = "../chains" }` 可解析
- 已安装 Rust / Cargo（建议 `rustup`）
- 网络可访问 Tron 与 EVM 公共 RPC（如 Nile、Sepolia、Arbitrum Sepolia）

## 代码结构

```
src/
├── main.rs    # 入口、CLI 分发、TRX 余额、Tron 交易监听
├── config.rs  # 环境与网络配置（TRON_NETWORK / EVM_NETWORK、RPC 选取）
├── trc20.rs   # Tron：TRX/TRC20 查询、构建、签名广播、全流程
└── erc20.rs   # EVM：原生 ETH / ERC20 查询、构建、签名广播、全流程
```

- **main.rs**：解析子命令，调用 `trc20` / `erc20` 模块；实现 `tron-balance`、`tron-monitor` 及 `help`
- **config.rs**：从环境变量解析 `TRON_NETWORK` / `EVM_NETWORK`，提供示例地址与 EVM RPC 健康检查
- **trc20.rs**：TRC20 只读、TRX 转账、TRC20 全流程、验证 TRC20 API
- **erc20.rs**：原生 ETH 余额/转账/监听、ERC20 只读/全流程、验证 ERC20 API

## 网络选择

| 环境变量       | 说明 | 可选值 |
|----------------|------|--------|
| `TRON_NETWORK` | Tron 网络 | `nile`（默认）、`mainnet`、`shasta` |
| `EVM_NETWORK`  | EVM 网络  | `sepolia`（默认）、`arbitrum-sepolia`、`arbitrum-one`、`mainnet` |

未设置 `EVM_RPC_URL` 时，程序从 SDK 提供的该网络备选 RPC 中依次健康检查选取可用节点。

## 命令一览

### Tron（网络由 TRON_NETWORK 指定）

| 命令 | 说明 |
|------|------|
| `tron-balance` | 查询 TRX 余额 |
| `tron-trc20` | TRC20 代币信息 + 构建转账（不签名不广播） |
| `tron-usdt-balance` | 查询 USDT 余额（按当前网络 USDT 合约） |
| `tron-verify-trc20` | 按 SDK 验证全部 TRC20 API |
| `tron-transfer` | TRX 原生转账：构建→签名→广播→监听 |
| `tron-full-flow` | 全自动 TRC20：构建→签名→广播→监听 |
| `tron-monitor` | 按交易哈希监听 Tron 交易（需 `TX_HASH`） |

### EVM 原生 ETH（网络由 EVM_NETWORK 指定）

| 命令 | 说明 |
|------|------|
| `eth-balance` | 查询原生 ETH 余额 |
| `eth-transfer` | 原生 ETH 转账全流程：构建→签名→广播→监听 |
| `eth-monitor` | 按交易哈希监听交易（需 `TX_HASH`） |

### ERC20（网络由 EVM_NETWORK 指定）

| 命令 | 说明 |
|------|------|
| `erc20-demo` | ERC20 代币信息 + 构建转账（不签名不广播） |
| `erc20-verify` | 按 SDK 验证全部 ERC20 API |
| `erc20-full-flow` | 全自动 ERC20：构建→签名→广播→监听 |

### 其他

| 命令 | 说明 |
|------|------|
| `help` / `-h` / `--help` | 显示命令列表与用法 |

查看所有命令：`cargo run -- help`

## 快速示例

```bash
# TRX 余额（Nile）
cargo run -- tron-balance

# TRC20 代币信息（可设置 TRC20_CONTRACT_ADDRESS）
export TRC20_CONTRACT_ADDRESS=TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf
cargo run -- tron-trc20

# TRX 原生转账（需私钥）
export TRON_PRIVATE_KEY=<64位十六进制私钥>
export TRON_TO_ADDRESS=TPsXm9mBMn8WGoDQcvroGPQbb3WpP7K15t
cargo run -- tron-transfer

# 全自动 TRC20（需私钥 + 合约）
export TRON_PRIVATE_KEY=<64位十六进制私钥>
export TRC20_CONTRACT_ADDRESS=<Nile 上的 TRC20 合约>
cargo run -- tron-full-flow

# 监听 Tron 交易
export TX_HASH=<交易哈希>
cargo run -- tron-monitor

# 原生 ETH 余额（Sepolia）
cargo run -- eth-balance

# 全自动 ERC20（需私钥）
export ETH_PRIVATE_KEY=<64位十六进制私钥>
export ERC20_AMOUNT=120
cargo run -- erc20-full-flow
```

**安全提示：** 私钥仅用于本地签名，不会上传；建议仅在测试网使用，勿泄露私钥。

## 文档

- **[docs/测试说明.md](docs/测试说明.md)** — 各命令的环境变量、预期结果与推荐测试顺序
- **[docs/功能与代码结构.md](docs/功能与代码结构.md)** — SDK 依赖、模块职责与功能说明
