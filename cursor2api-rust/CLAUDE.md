# cursor2api-rust

## 项目说明

Rust Axum Web UI 后端，为 cursor2api 提供 Web 管理界面。

## 运行

```bash
# 开发模式（读取 cursor2api 配置和日志）
cargo run

# 自定义路径
CONFIG_PATH=/path/to/config.yaml DB_PATH=/path/to/cursor2api.db LOG_DIR=/path/to/logs cargo run
```

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `PORT` | `3001` | 服务端口 |
| `CONFIG_PATH` | `../cursor2api/config.yaml` | config.yaml 路径 |
| `DB_PATH` | `../cursor2api/logs/cursor2api.db` | SQLite 路径 |
| `LOG_DIR` | `../cursor2api/logs` | JSONL 日志目录 |
| `AUTH_TOKENS` | `` | 逗号分隔的鉴权 token 列表 |

## API 端点

| 路由 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/api/config` | GET | 读取配置 |
| `/api/config` | POST | 保存配置 |
| `/api/logs` | GET | 日志条目（支持过滤）|
| `/api/logs/clear` | POST | 清空日志 |
| `/api/logs/stream` | GET | SSE 实时日志流 |
| `/api/requests` | GET | 请求摘要列表 |
| `/api/requests/more` | GET | 分页加载更多 |
| `/api/vue/stats` | GET | 统计数据 |
| `/api/payload/:id` | GET | 请求完整 payload |
| `/*` | GET | Vue UI 静态文件 |

## Vue UI 构建

```bash
# 构建到 Rust static 目录
cd ../cursor2api/vue-ui && RUST_UI=1 npm run build

# 开发模式（代理到 Rust 3001）
npm run dev
```

## 模块结构

- `main.rs`：路由注册、服务启动
- `state.rs`：`AppState`、`ServerConfig`
- `types.rs`：对应 TypeScript 类型定义
- `config.rs`：读写 config.yaml
- `logger.rs`：SQLite + JSONL 日志读取
- `auth.rs`：Bearer token 鉴权中间件
- `routes/config.rs`：配置 API
- `routes/logs.rs`：日志 API
- `routes/sse.rs`：SSE 实时流 + 文件监听
