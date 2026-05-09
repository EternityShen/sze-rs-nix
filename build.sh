#!/usr/bin/env bash

set -euo pipefail

# 安装 Rust 组件
echo "[+] 安装 Rust 组件..."
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
cargo install bpf-linker
# 使用稳定版工具链添加 android 目标
rustup target add aarch64-linux-android

echo "[✓] Rust 组件安装完成"

# 检查是否安装了 cargo-ndk
if ! command -v cargo-ndk &> /dev/null; then
    echo "[+] 安装 cargo-ndk..."
    cargo install cargo-ndk
else
    echo "[✓] cargo-ndk 已安装"
fi

echo "[✓] 所有编译工具已准备就绪"
echo "========================================"
echo

echo "[+] 正在编译 eBPF 程序（目标平台：bpfel-unknown-none，nightly 工具链）"
cargo +nightly build --manifest-path ebpf/Cargo.toml --target bpfel-unknown-none --release -Z build-std=core

echo "[+] 正在编译用户态程序"
echo "[+] 正在编译用户态程序（本机平台，发布模式）"
cargo build --manifest-path user/Cargo.toml --release

echo "[+] 正在编译用户态程序（Android ARM64 平台，发布模式）"
cargo build --manifest-path user/Cargo.toml --target aarch64-linux-android --release

EBPF_OBJ=target/bpfel-unknown-none/release/libebpf.so
USER_BIN=target/release/sze-rs-nix
AARCH64_USER_BIN=target/aarch64-linux-android/release/sze-rs-nix

echo "[+] 正在打包模块"
MODDIR=mode
MODNAME=sze-rs-nix-release-3.0
cp ${EBPF_OBJ} ${MODDIR}/system/lib/
cp ${AARCH64_USER_BIN} ${MODDIR}/system/bin/

cd ${MODDIR}

now=$(date +%Y-%m-%d_%H-%M-%S)
mode_versionCode=$(date +%Y%m%d)

sed -i "3s/.*/versionCode=${mode_versionCode}/" "module.prop"
sed -i "s/\"versionCode\": [0-9]*,/\"versionCode\": ${mode_versionCode},/" "vtools/powercfg.json"

zip -r ${MODNAME}.zip ./*

echo
echo "构建完成。"
echo "eBPF 动态库文件：${EBPF_OBJ}"
echo "x86_64 用户态可执行文件：${USER_BIN}"
echo "AArch64 用户态可执行文件：${AARCH64_USER_BIN}"
echo "模块：${MODNAME}.zip"
echo "构建日期：${now}"

echo "OvO  ShenEternity @ ${now}"
