#!/bin/bash
# Rust 开发热重启脚本
# 依赖：cargo install cargo-watch
set -e
cargo watch -x run
