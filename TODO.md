# cursor2api-rust TODO

## Phase 0: 准备工作 ✅
- [x] T0.1 cursor2api 创建 git backup 分支 `backup/original`
- [x] T0.2 cursor-web/ 根目录初始化 git
- [x] T0.3 创建 CLAUDE.md
- [x] T0.4 创建 TODO.md

## Phase 1: Rust 项目初始化
- [ ] T1.1 `cargo new cursor2api-rust`
- [ ] T1.2 配置 Cargo.toml
- [ ] T1.3 基础路由骨架和 CORS 中间件
- [ ] T1.4 健康检查 `/health`
- [ ] T1.5 自测：cargo build 通过，curl /health 返回 200
- [ ] T1.6 git commit

## Phase 2: 配置管理 API
- [ ] T2.1 config.rs：读取 config.yaml
- [ ] T2.2 GET `/api/config`
- [ ] T2.3 POST `/api/config`
- [ ] T2.4 自测
- [ ] T2.5 git commit

## Phase 3: 日志读取 API
- [ ] T3.1 logger.rs：读取 JSONL
- [ ] T3.2 SQLite 日志读取
- [ ] T3.3 GET `/api/requests`
- [ ] T3.4 GET `/api/logs`
- [ ] T3.5 GET `/api/vue/stats`
- [ ] T3.6 GET `/api/payload/:requestId`
- [ ] T3.7 POST `/api/logs/clear`
- [ ] T3.8 GET `/api/requests/more`
- [ ] T3.9 自测
- [ ] T3.10 git commit

## Phase 4: SSE 实时日志流
- [ ] T4.1 GET `/api/logs/stream` SSE
- [ ] T4.2 文件/SQLite 变更监听
- [ ] T4.3 自测
- [ ] T4.4 git commit

## Phase 5: Vue UI 集成
- [ ] T5.1 构建 vue-ui
- [ ] T5.2 静态文件服务
- [ ] T5.3 SPA fallback
- [ ] T5.4 调整 vite 代理配置
- [ ] T5.5 自测
- [ ] T5.6 git commit

## Phase 6: 鉴权中间件
- [ ] T6.1 Bearer token 鉴权
- [ ] T6.2 支持 query param `?token=xxx`
- [ ] T6.3 自测
- [ ] T6.4 git commit

## Phase 7: 完整集成测试
- [ ] T7.1 同时启动两服务
- [ ] T7.2 验证读取同一 config.yaml 和 SQLite
- [ ] T7.3 Vue UI 正常显示
- [ ] T7.4 配置保存后 Node.js 热重载
- [ ] T7.5 打 tag v0.1.0-rust-ui
