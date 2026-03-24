# cursor2api-rust TODO

## Phase 0: 准备工作 ✅
- [x] T0.1 cursor2api 创建 git backup 分支 `backup/original`
- [x] T0.2 cursor-web/ 根目录初始化 git
- [x] T0.3 创建 CLAUDE.md
- [x] T0.4 创建 TODO.md

## Phase 1: Rust 项目初始化 ✅
- [x] T1.1 `cargo new cursor2api-rust`
- [x] T1.2 配置 Cargo.toml
- [x] T1.3 基础路由骨架和 CORS 中间件
- [x] T1.4 健康检查 `/health`
- [x] T1.5 自测：cargo build 通过，curl /health 返回 200
- [x] T1.6 git commit

## Phase 2: 配置管理 API ✅
- [x] T2.1 config.rs：读取 config.yaml
- [x] T2.2 GET `/api/config`
- [x] T2.3 POST `/api/config`
- [x] T2.4 自测
- [x] T2.5 git commit

## Phase 3: 日志读取 API ✅
- [x] T3.1 logger.rs：读取 JSONL
- [x] T3.2 SQLite 日志读取
- [x] T3.3 GET `/api/requests`
- [x] T3.4 GET `/api/logs`
- [x] T3.5 GET `/api/vue/stats`
- [x] T3.6 GET `/api/payload/:requestId`
- [x] T3.7 POST `/api/logs/clear`
- [x] T3.8 GET `/api/requests/more`
- [x] T3.9 自测
- [x] T3.10 git commit

## Phase 4: SSE 实时日志流 ✅
- [x] T4.1 GET `/api/logs/stream` SSE
- [x] T4.2 文件变更监听（notify crate）
- [x] T4.3 自测
- [x] T4.4 git commit

## Phase 5: Vue UI 集成 ✅
- [x] T5.1 构建 vue-ui（RUST_UI=1）
- [x] T5.2 静态文件服务（ServeDir）
- [x] T5.3 SPA fallback
- [x] T5.4 调整 vite 代理配置（dev → 3001）
- [x] T5.5 自测（curl / 返回 200）
- [x] T5.6 git commit

## Phase 6: 鉴权中间件 ✅
- [x] T6.1 Bearer token 鉴权中间件
- [x] T6.2 支持 query param `?token=xxx`
- [x] T6.3 自测
- [x] T6.4 git commit

## Phase 7: 完整集成测试 ✅
- [x] T7.1 /health, /api/config, / 全部通过
- [x] T7.2 验证读取 config.yaml 和 SQLite
- [x] T7.3 Vue UI 正常显示（HTTP 200）
- [ ] T7.4 配置保存后 Node.js 热重载（需运行 Node.js 验证）
- [x] T7.5 打 tag v0.1.0-rust-ui
