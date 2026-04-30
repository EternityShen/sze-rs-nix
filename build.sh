#!/usr/bin/env bash

set -euo pipefail

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
MODNAME=sze-rs-nix-release-1.0
cp ${EBPF_OBJ} ${MODDIR}/system/lib/
cp ${AARCH64_USER_BIN} ${MODDIR}/system/bin/

cd ${MODDIR}

now=$(date +%Y-%m-%d_%H-%M-%S)
mode_versionCode=$(date +%Y%m%d)

sed -i "3s/.*/versionCode=${mode_versionCode}/" "module.prop"
sed -i "s/\"versionCode\": [0-9]*,/\"versionCode\": ${mode_versionCode},/" "vtools/powercfg.json"

rm ${MODNAME}.zip
zip -r ${MODNAME}.zip ./*

echo
echo "构建完成。"
echo "eBPF 动态库文件：${EBPF_OBJ}"
echo "x86_64 用户态可执行文件：${USER_BIN}"
echo "AArch64 用户态可执行文件：${AARCH64_USER_BIN}"
echo "模块：${MODNAME}.zip"
echo "构建日期：${now}"

echo "OvO  ShenEternity @ ${now}"
