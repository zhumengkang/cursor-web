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

## Phase 8: API Key 管理面板 ✅
- [x] T8.1 新建 `keys.db`（SQLite，`KEYS_DB_PATH` 环境变量，默认 `./keys.db`）
- [x] T8.2 `src/keys_db.rs`：init/list（key_value 脱敏前4后4）/create/update/delete
- [x] T8.3 `src/routes/keys.rs`：CRUD 路由 + stats 聚合
  - GET `/api/keys` 获取列表
  - POST `/api/keys` 新建
  - PUT `/api/keys/:id` 编辑
  - DELETE `/api/keys/:id` 删除
  - GET `/api/keys/stats` 全局 token 统计
- [x] T8.4 stats 支持多维度时间筛选
  - `since` / `until` 参数（unix ms）
  - `granularity` 参数：`hour` / `day` / `week` / `month`
  - SQLite 动态 GROUP BY 表达式（本地时间）
- [x] T8.5 Vue `src/api.ts` 新增 keys CRUD + stats fetch 函数
- [x] T8.6 Vue `src/types.ts` 新增 ApiKey / KeyStats / ModelBreakdown / DailyBreakdown
- [x] T8.7 Vue `KeysPanel.vue` 完整管理面板
  - 统计卡片（总请求/成功/失败/Token 消耗/成功率/平均响应时间）
  - 时间范围快捷按钮：全部 / 今天 / 近7天 / 近30天 / 本月
  - 粒度切换：小时 / 天 / 周 / 月
  - 模型分布表格
  - 用量趋势表格（按所选粒度）
  - API Key 列表（增删改查、启用/禁用）
- [x] T8.8 `App.vue` 添加 Tab 导航（请求日志 / API Keys）
- [x] T8.9 Rust cargo build 通过（无错误）

## Phase 9: 代理设置 ✅
- [x] T9.1 Node.js 安装 `socks-proxy-agent` 依赖
- [x] T9.2 重写 `proxy-agent.ts`，支持全协议：
  - `http://[user:pass@]host:port`
  - `https://[user:pass@]host:port`
  - `socks4://[user:pass@]host:port`
  - `socks4a://[user:pass@]host:port`
  - `socks5://[user:pass@]host:port`
  - `socks5h://[user:pass@]host:port`
  - http/https 代理使用 `undici.ProxyAgent`（原生支持）
  - SOCKS 代理使用 `SocksProxyAgent` 替换全局 Node.js agent
  - 缓存机制：proxy URL 变化时自动重新创建 agent
- [x] T9.3 `config-api.ts` 新增 proxy 字段读取和保存
  - GET `/api/config` 返回 `proxy` 字段
  - POST `/api/config` 支持保存/清空 proxy
- [x] T9.4 Vue `types.ts` HotConfig 新增 `proxy?: string`
- [x] T9.5 Vue `ConfigDrawer.vue` 新增「代理设置」Group
  - 支持输入完整代理 URL（含用户名密码）
  - 留空表示直连
  - 描述说明支持的格式

## 待办 / 改进项
- [ ] 代理连通性测试按钮（在 ConfigDrawer 里发一个 /api/proxy/test 请求）
- [ ] API Key 页面：每个 Key 的独立使用记录（需 Node.js 侧记录 which-key，当前约束不改 Node.js）
- [ ] 统计页面图表化（折线图/柱状图替换纯表格）
- [ ] T7.4 配置保存后 Node.js 热重载验证
- [ ] Docker Compose 一键启动文档
