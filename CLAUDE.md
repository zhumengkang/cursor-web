# cursor-web Monorepo 约定

## 项目结构

```
cursor-web/
├── cursor2api/          # 原 Node.js 项目（不修改核心逻辑）
└── cursor2api-rust/     # Rust Axum Web UI 后端
```

## 职责划分

- **cursor2api (Node.js, 默认端口 3000)**：AI 代理核心，处理 `/v1/messages`、`/v1/chat/completions` 等
- **cursor2api-rust (Rust, 默认端口 3001)**：Web UI 后端，处理 `/api/*` 和静态文件服务

## Rust 后端关键约定

- 框架：Axum + Tokio
- 配置文件：读取 `../cursor2api/config.yaml`（相对路径，可通过 `CONFIG_PATH` 环境变量覆盖）
- SQLite 日志：读取 `../cursor2api/logs/cursor2api.db`（可通过 `DB_PATH` 覆盖）
- JSONL 日志目录：读取 `../cursor2api/logs/`（可通过 `LOG_DIR` 覆盖）
- 静态文件：`static/` 目录（Vue UI build 产物）

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `PORT` | `3001` | Rust 服务端口 |
| `CONFIG_PATH` | `../cursor2api/config.yaml` | config.yaml 路径 |
| `DB_PATH` | `../cursor2api/logs/cursor2api.db` | SQLite 路径 |
| `LOG_DIR` | `../cursor2api/logs` | JSONL 日志目录 |

## Vue UI 开发代理

开发模式下，`vue-ui/vite.config.ts` 代理 `/api/*` 到 `http://localhost:3001`。

## Git 分支策略

- `main`：稳定版本
- `backup/original`（在 cursor2api 子仓库）：Node.js 原始代码备份
