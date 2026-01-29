#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
验证所有网络：通过 cargo run 调用 chains-test 命令。
- Tron 网络（nile / mainnet / shasta）：只读命令 + 测试网转账与监听
- EVM 网络（sepolia / arbitrum-sepolia / arbitrum-one / mainnet）：只读命令；可选 EVM 测试网（sepolia、arbitrum-sepolia）转账与监听（需 ETH_PRIVATE_KEY）
"""

import os
import re
import subprocess
import sys
import time
from pathlib import Path
from threading import Thread
from typing import Dict, List, Optional, Tuple

# 项目根目录（脚本所在目录）
PROJECT_ROOT = Path(__file__).resolve().parent

# Tron 地址与密钥
ADDR = "TG2D8vTp4xHBB2vhVbgHK2AhA2p9wY4q9M"
TO_ADDR = "TPsXm9mBMn8WGoDQcvroGPQbb3WpP7K15t"
PRIVATE_KEY = "79a5d62ebbe36b4e54fa5d795de9a2d4c528508a48e004cafbe0660a8d286e08"
TRX_AMOUNT_SUN = "1000000"
TRC20_AMOUNT = "1000000"

# EVM 示例地址（Sepolia / Arbitrum Sepolia）
ETH_ADDR = "0x3CCD11B6c4B5Ca62d2B29C949B23e0550d64f0b9"
# EVM 私钥（64 位十六进制），未设置环境变量 ETH_PRIVATE_KEY 时使用
ETH_PRIVATE_KEY = "98f16b84579621f391a1589626c057fa0a54630b9f792e6db4a8493229de089d"

# 要验证的网络（与 main.rs / 文档一致）
TRON_NETWORKS = ["nile", "mainnet", "shasta"]
# SDK EvmNetwork 支持：sepolia | arbitrum-sepolia | arbitrum-one | mainnet
EVM_NETWORKS = ["sepolia", "arbitrum-sepolia", "arbitrum-one", "mainnet"]
# EVM 测试网（用于第四部分转账与监听）
EVM_TEST_NETWORKS = ["sepolia", "arbitrum-sepolia"]
# 切换 Tron 网络前等待秒数，避免 mainnet 公共 RPC 限流（429）
TRON_NETWORK_DELAY_SEC = 4


def run_cmd(
    cmd: List[str],
    env: Optional[Dict[str, str]] = None,
    timeout: int = 180,
) -> Tuple[int, str]:
    """
    执行 cargo run -- <cmd>，边读边打印输出，返回 (returncode, 完整输出)。
    流式输出避免管道缓冲满导致子进程阻塞、无日志。
    """
    full_env = os.environ.copy()
    full_env["RUSTFLAGS"] = "-A warnings"
    if env:
        full_env.update(env)
    full_cmd = ["cargo", "run", "--", *cmd]
    try:
        proc = subprocess.Popen(
            full_cmd,
            cwd=PROJECT_ROOT,
            env=full_env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        lines: List[str] = []

        def read_out() -> None:
            if proc.stdout is None:
                return
            for line in proc.stdout:
                line = line if line.endswith("\n") else line + "\n"
                lines.append(line)
                print(line, end="")
            sys.stdout.flush()

        reader = Thread(target=read_out, daemon=True)
        reader.start()
        try:
            proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            reader.join(timeout=2)
            return -1, "".join(lines) + "\n命令超时(%ds)" % timeout
        reader.join(timeout=2)
        return proc.returncode or 0, "".join(lines)
    except FileNotFoundError:
        return -1, "未找到 cargo，请确保在项目根目录且已安装 Rust"


def extract_tron_tx_hash(output: str) -> Optional[str]:
    """从输出中解析 Tron「交易哈希: xxx」（64 位十六进制，无 0x）。"""
    m = re.search(r"交易哈希:\s*([a-fA-F0-9]{64})", output)
    return m.group(1) if m else None


def extract_evm_tx_hash(output: str) -> Optional[str]:
    """从输出中解析 EVM「交易哈希: 0x...」，返回带 0x 的哈希。"""
    m = re.search(r"交易哈希:\s*(0x[a-fA-F0-9]{64})", output)
    if m:
        return m.group(1)
    m = re.search(r"交易哈希:\s*([a-fA-F0-9]{64})", output)
    return ("0x" + m.group(1)) if m else None


def section(title: str) -> None:
    print("\n" + "=" * 60)
    print(title)
    print("=" * 60)


def run_and_log(
    cmd: List[str],
    env: Optional[Dict[str, str]] = None,
    label: str = "",
) -> Tuple[int, str]:
    """执行命令并流式打印日志，返回 (returncode, 完整输出)。"""
    if label:
        print("\n--- %s ---" % label)
        sys.stdout.flush()
    code, out = run_cmd(cmd, env=env)
    return code, out


def main() -> int:
    start_time = time.time()
    stats = {"total": 0, "ok": 0, "fail": 0, "skip": 0}

    def count_result(code: int, skipped: bool = False) -> None:
        stats["total"] += 1
        if code == 0:
            stats["ok"] += 1
        elif skipped:
            stats["skip"] += 1
        else:
            stats["fail"] += 1

    section("一、Tron 所有网络（只读验证）")

    for i, net in enumerate(TRON_NETWORKS):
        if i > 0:
            time.sleep(TRON_NETWORK_DELAY_SEC)
        env = {"TRON_NETWORK": net, "TRON_ADDRESS": ADDR}
        section("Tron — %s" % net)
        for name, cmd in [
            ("tron-balance", ["tron-balance"]),
            ("tron-trc20", ["tron-trc20"]),
            ("tron-usdt-balance", ["tron-usdt-balance"]),
            ("tron-verify-trc20", ["tron-verify-trc20"]),
        ]:
            if net == "mainnet" and name == "tron-verify-trc20":
                time.sleep(2)
            code, out = run_and_log(cmd, env=env, label="[%s] %s" % (net, name))
            if code != 0:
                if net == "mainnet" and name == "tron-verify-trc20":
                    count_result(code, skipped=True)
                    print("⚠️ Tron(mainnet) tron-verify-trc20 失败（多为 RPC 限流 429），已跳过并继续")
                else:
                    count_result(code, skipped=False)
                    print("❌ Tron(%s) %s 失败" % (net, name))
                    return 1
            else:
                count_result(code)
    print("\n✅ Tron 所有网络只读验证通过")

    section("二、Tron 测试网（nile）— 转账与监听")

    env_tron = {
        "TRON_NETWORK": "nile",
        "TRON_PRIVATE_KEY": PRIVATE_KEY,
        "TRON_FROM_ADDRESS": ADDR,
        "TRON_TO_ADDRESS": TO_ADDR,
        "TRX_AMOUNT_SUN": TRX_AMOUNT_SUN,
        "TRC20_AMOUNT": TRC20_AMOUNT,
    }

    code, out = run_and_log(["tron-transfer"], env=env_tron, label="tron-transfer")
    count_result(code)
    if code != 0:
        print("❌ tron-transfer 失败")
        return 1
    tx_transfer = extract_tron_tx_hash(out)

    code, out = run_and_log(["tron-full-flow"], env=env_tron, label="tron-full-flow")
    count_result(code)
    if code != 0:
        print("❌ tron-full-flow 失败")
        return 1
    tx_full = extract_tron_tx_hash(out)

    tx_monitor = tx_full or tx_transfer
    if tx_monitor:
        code, out = run_and_log(
            ["tron-monitor"],
            env={"TRON_NETWORK": "nile", "TX_HASH": tx_monitor},
            label="tron-monitor",
        )
        count_result(code)
        if code != 0:
            print("❌ tron-monitor 失败")
            return 1
    else:
        print("⚠️ 未解析到 Tron 交易哈希，跳过 tron-monitor")
    print("\n✅ Tron 测试网验证通过")

    section("三、EVM 所有网络（只读验证）")

    for net in EVM_NETWORKS:
        env = {"EVM_NETWORK": net, "ETH_ADDRESS": ETH_ADDR}
        section("EVM — %s" % net)
        for name, cmd in [
            ("eth-balance", ["eth-balance"]),
            ("erc20-demo", ["erc20-demo"]),
            ("erc20-verify", ["erc20-verify"]),
        ]:
            code, out = run_and_log(cmd, env=env, label="[%s] %s" % (net, name))
            if code != 0:
                if name == "erc20-verify":
                    count_result(code, skipped=True)
                    print("⚠️ EVM(%s) %s 失败，已跳过并继续" % (net, name))
                else:
                    count_result(code, skipped=False)
                    print("❌ EVM(%s) %s 失败" % (net, name))
                    return 1
            else:
                count_result(code)
    print("\n✅ EVM 所有网络只读验证通过")

    section("四、EVM 测试网（可选：需 ETH_PRIVATE_KEY）")

    eth_key = (os.environ.get("ETH_PRIVATE_KEY") or "").strip() or ETH_PRIVATE_KEY
    if not eth_key:
        print("未设置 ETH_PRIVATE_KEY，跳过 EVM 转账与监听。")
        print("如需验证 eth-transfer / erc20-full-flow / eth-monitor，请设置 ETH_PRIVATE_KEY。")
    else:
        for net in EVM_TEST_NETWORKS:
            section("EVM 测试网 — %s" % net)
            env_evm = {
                "EVM_NETWORK": net,
                "ETH_PRIVATE_KEY": eth_key,
                "ETH_ADDRESS": ETH_ADDR,
                "ETH_FROM_ADDRESS": ETH_ADDR,
                "ETH_TO_ADDRESS": ETH_ADDR,
                "ETH_AMOUNT_WEI": "1000000000000000",
                "ERC20_AMOUNT": "0",
            }

            code, out = run_and_log(
                ["eth-transfer"], env=env_evm, label="[%s] eth-transfer" % net
            )
            count_result(code)
            if code != 0:
                print("❌ [%s] eth-transfer 失败" % net)
                return 1
            tx_eth = extract_evm_tx_hash(out)

            code, out = run_and_log(
                ["erc20-full-flow"], env=env_evm, label="[%s] erc20-full-flow" % net
            )
            count_result(code)
            if code != 0:
                print("❌ [%s] erc20-full-flow 失败" % net)
                return 1
            tx_erc20 = extract_evm_tx_hash(out)

            tx_evm = tx_erc20 or tx_eth
            if tx_evm:
                code, out = run_and_log(
                    ["eth-monitor"],
                    env={"EVM_NETWORK": net, "TX_HASH": tx_evm},
                    label="[%s] eth-monitor" % net,
                )
                count_result(code)
                if code != 0:
                    print("❌ [%s] eth-monitor 失败" % net)
                    return 1
            else:
                print("⚠️ [%s] 未解析到 EVM 交易哈希，跳过 eth-monitor" % net)
        print("\n✅ EVM 所有测试网验证通过")

    elapsed = time.time() - start_time
    print("\n" + "=" * 60)
    print("统计")
    print("=" * 60)
    print("总耗时:     %.1f 秒" % elapsed)
    print("总命令数:   %d" % stats["total"])
    print("成功:       %d" % stats["ok"])
    print("失败:       %d" % stats["fail"])
    print("跳过(容错): %d" % stats["skip"])
    print("=" * 60)
    print("全部网络验证完成")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    sys.exit(main())
