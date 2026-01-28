# chains-test

一个用于**本地测试 `chains-sdk` 的示例 Rust 项目**。

## 前置条件

- 已在同级目录下克隆/存在 `chains-sdk` 项目：

  - 当前项目路径：`/Users/f/Documents/my/code/chains-test`
  - SDK 项目路径：`/Users/f/Documents/my/code/chains`

- 已安装 Rust 与 Cargo（建议使用 `rustup` 安装）。

## 运行示例

在 `chains-test` 目录下执行不同子命令：

- **查询 TRX 余额**

  ```bash
  cargo run -- balance
  ```

- **TRC20 代币信息与构建转账交易（不签名不广播）**

  先设置代币合约地址环境变量：

  ```bash
  export TRC20_CONTRACT_ADDRESS=<TRC20合约地址>
  cargo run -- trc20
  ```

- **TRX 原生转账（构建 → 签名 → 广播 → 监听）**

  ```bash
  export TRON_PRIVATE_KEY=<64位十六进制私钥>
  # 可选：TRON_FROM_ADDRESS、TRON_TO_ADDRESS、TRX_AMOUNT_SUN（默认 1000 sun）
  cargo run -- trx-transfer
  ```

- **监听交易确认状态**

  先设置要监听的交易哈希：

  ```bash
  export TX_HASH=<交易哈希>
  cargo run -- monitor
  ```

- **发送交易并等待确认（示例用假交易数据）**

  ```bash
  cargo run -- send-demo
  ```

  > 注意：`send-demo` 示例中使用的是 SDK 示例里的假 `raw_tx` JSON，仅用于演示接口调用流程。  
  > 真正上链前，请务必替换为你自己“构建并签名”的原始交易数据。

- **真实上链：发送你已签名的原始交易 JSON**

  方式一：直接用环境变量传 JSON（适合短交易）：

  ```bash
  export SIGNED_TX_JSON='{"raw_data":{...},"signature":[...]}'
  cargo run -- send-signed
  ```

  方式二：从文件读取（推荐，便于调试和保存）：

  ```bash
  echo '{"raw_data":{...},"signature":[...]}' > signed_tx.json
  export SIGNED_TX_PATH=$(pwd)/signed_tx.json
  cargo run -- send-signed
  ```

  chains-sdk 会：

  - 使用 Tron Nile 测试网的 RPC 发送该已签名交易
  - 打印交易哈希
  - 轮询等待确认结果（成功 / 失败 / 超时）

- **方案 B：全自动流程（构建 → 私钥签名 → 广播 → 监听）**

  一条命令完成 TRC20 转账：在本项目内用环境变量传入私钥与合约等，自动完成构建、签名、广播、监听。

  **必填环境变量：**

  - `TRON_PRIVATE_KEY`：发送方**私钥的十六进制**（32 字节 = 64 个 `0-9`/`a-f` 字符，可带 `0x` 前缀）。**不能填助记词**；若只有助记词，请先用 TronLink 或 `tronweb` 等从助记词导出私钥 hex 再填入。
  - `TRC20_CONTRACT_ADDRESS`：TRC20 合约地址（**必须是当前网络上的合约**：`full-flow` 使用 Nile 测试网，请使用在 Nile 上已部署的 TRC20 合约地址，不要填主网合约如 USDC，否则会报 `triggerSmartContract missing transaction`）

  **可选环境变量：**

  - `TRON_FROM_ADDRESS`：发送方地址（默认示例地址）
  - `TRON_TO_ADDRESS`：接收方地址（默认同 FROM）
  - `TRC20_AMOUNT`：转账金额，最小单位（默认 `1000000`）
  - `TRC20_FEE_LIMIT`：费用上限 sun（默认 `100000000`）

  **示例（Nile 测试网，请将合约改为你在 Nile 上部署或已有的 TRC20 合约）：**

  ```bash
  export TRON_PRIVATE_KEY=79a5d62ebbe36b4e54fa5d795de9a2d4c528508a48e004cafbe0660a8d286e08
  export TRC20_CONTRACT_ADDRESS=TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf
  export TRON_FROM_ADDRESS=TG2D8vTp4xHBB2vhVbgHK2AhA2p9wY4q9M
  export TRON_TO_ADDRESS=TPsXm9mBMn8WGoDQcvroGPQbb3WpP7K15t
  cargo run -- full-flow
  ```

  **安全提示：** 私钥仅用于本地签名，不会上传；建议仅在测试网（如 Nile）使用，勿在生产环境泄露私钥。
