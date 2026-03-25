<template>
  <header class="app-header">
    <div class="header-left">
      <h1 class="brand"><span class="brand-kk">KK</span><span class="brand-dot">·</span><span class="brand-proxy">PROXY</span></h1>
    </div>
    <div class="header-center">
      <div class="stats-pills">
        <div class="sc" title="总请求数">
          <svg class="sc-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 8h12M8 2l6 6-6 6"/></svg>
          <b>{{ stats.totalRequests }}</b>
        </div>
        <div class="sc sc-ok" title="成功完成的请求数">
          <svg class="sc-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 8l4 4 6-7"/></svg>
          <b>{{ stats.successCount }}</b>
        </div>
        <div class="sc sc-deg" title="降级请求数（重试后成功）">
          <svg class="sc-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><circle cx="8" cy="8" r="6"/><path d="M8 5v4M8 11v.5"/></svg>
          <b>{{ stats.degradedCount }}</b>
        </div>
        <div class="sc sc-err" title="失败请求数">
          <svg class="sc-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><circle cx="8" cy="8" r="6"/><path d="M5.5 5.5l5 5M10.5 5.5l-5 5"/></svg>
          <b>{{ stats.errorCount }}</b>
        </div>
        <div class="sc" v-if="stats.avgResponseTime" title="平均响应时间">
          <svg class="sc-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="8" cy="8" r="6"/><path d="M8 5v3.5l2.5 1.5"/></svg>
          <b>{{ fmtMs(stats.avgResponseTime) }}</b>
        </div>
        <div class="sc" v-if="stats.avgTTFT" title="平均首 Token 时间（TTFT）">
          <svg class="sc-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3 8l3-4v3h4V4l3 4-3 4v-3H6v3z"/></svg>
          <b>{{ fmtMs(stats.avgTTFT) }}</b> <span class="sc-label">TTFT</span>
        </div>
      </div>
    </div>
    <div class="header-right">
      <button class="hdr-btn copy-btn" @click="onCopyEndpoint" :title="copyTip">
        <svg v-if="!copied" width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="5" y="5" width="9" height="9" rx="2"/><path d="M11 5V3a2 2 0 0 0-2-2H3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h2"/></svg>
        <svg v-else width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M2 8l4 5 8-9"/></svg>
        {{ copied ? '已复制' : '复制端点' }}
      </button>
      <button v-if="loggedIn && authStore.token" class="hdr-btn logout-btn" @click="onLogout" title="退出登录">退出</button>
      <button class="hdr-btn config-btn" @click="emit('openConfig')" title="打开配置面板">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
        配置
      </button>
      <button class="hdr-btn clear-btn" @click="onClear" title="清空所有日志（不可恢复）">
        <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><polyline points="3,4 13,4"/><path d="M6 4V2h4v2M5 4v9a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1V4"/></svg>
        清空
      </button>
      <button class="hdr-btn theme-btn" @click="toggleTheme" :title="isDark ? '切换到浅色主题' : '切换到深色主题'">
        <svg v-if="isDark" width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="8" cy="8" r="3"/><path d="M8 1v1.5M8 13.5V15M1 8h1.5M13.5 8H15M3.2 3.2l1.1 1.1M11.7 11.7l1.1 1.1M3.2 12.8l1.1-1.1M11.7 4.3l1.1-1.1"/></svg>
        <svg v-else width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M13.5 10.5A6 6 0 0 1 5.5 2.5a6 6 0 1 0 8 8z"/></svg>
      </button>
      <div class="conn" :class="connected ? 'on' : 'off'">
        <div class="d" />
        <span>{{ connected ? '已连接' : '重连中…' }}</span>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useStatsStore } from '../stores/stats';
import { useLogsStore } from '../stores/logs';
import { useAuthStore } from '../stores/auth';
import { storeToRefs } from 'pinia';

defineProps<{ connected: boolean }>();
const emit = defineEmits<{ openConfig: [] }>();

const statsStore = useStatsStore();
const logsStore = useLogsStore();
const authStore = useAuthStore();
const { stats } = storeToRefs(statsStore);
const { loggedIn } = storeToRefs(authStore);

function fmtMs(ms: number): string {
  return ms >= 1000 ? (ms / 1000).toFixed(2).replace(/\.?0+$/, '') + 's' : ms + 'ms';
}

async function onLogout() {
  authStore.clearToken();
  // 检查无 token 时是否还能访问（open access 模式），能则不跳转登录页
  try {
    const res = await fetch('/api/vue/stats');
    if (res.ok) {
      // 服务端不需要授权，保持登录状态
      return;
    }
  } catch { /* ignore */ }
  authStore.loggedIn = false;
}

const isDark = ref(false);

onMounted(() => {
  isDark.value = (localStorage.getItem('cursor2api_theme') ?? 'light') === 'dark';
  applyTheme();
});

function applyTheme() {
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light');
}

function toggleTheme() {
  isDark.value = !isDark.value;
  localStorage.setItem('cursor2api_theme', isDark.value ? 'dark' : 'light');
  applyTheme();
}

async function onClear() {
  if (!confirm('确定清空所有日志？此操作不可恢复。')) return;
  await logsStore.clear();
  await statsStore.load();
}

const copied = ref(false);
async function onCopyEndpoint() {
  const base = `${location.protocol}//${location.hostname}:3000`;
  const text = `API Base URL: ${base}\nAnthropic: ${base}/v1/messages\nOpenAI: ${base}/v1/chat/completions`;
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
  } catch { /* ignore */ }
  setTimeout(() => { copied.value = false; }, 2000);
}
const copyTip = '复制 API 端点地址';
</script>

<style scoped>
.app-header {
  display: grid; grid-template-columns: 1fr auto 1fr;
  align-items: center;
  padding: 12px 20px;
  background:
    radial-gradient(ellipse 70% 120% at 50% -30%, rgba(96,165,250,0.12) 0%, transparent 65%),
    radial-gradient(ellipse 40% 80% at 0% 50%, rgba(167,139,250,0.06) 0%, transparent 60%),
    rgba(8,12,20,0.92);
  backdrop-filter: blur(24px) saturate(180%);
  -webkit-backdrop-filter: blur(24px) saturate(180%);
  border-bottom: 1px solid rgba(96,165,250,0.12);
  flex-shrink: 0; z-index: 100; position: relative;
  box-shadow: 0 1px 0 rgba(255,255,255,0.04), 0 4px 24px rgba(0,0,0,0.3);
}
[data-theme="light"] .app-header {
  background:
    radial-gradient(ellipse 70% 120% at 50% -30%, rgba(99,102,241,0.06) 0%, transparent 65%),
    rgba(255,255,255,0.88);
  border-bottom: 1px solid rgba(226,232,240,0.9);
  box-shadow: 0 1px 0 rgba(255,255,255,0.9), 0 4px 20px rgba(0,0,0,0.06);
}
[data-theme="light"] .sc {
  background: rgba(255,255,255,0.85);
  border-color: rgba(226,232,240,0.9);
  box-shadow: 0 1px 3px rgba(0,0,0,.04);
}
.header-left { display: flex; align-items: center; gap: 14px; }
.header-center { display: flex; justify-content: center; align-items: center; }
.brand {
  font-size: 18px; font-weight: 900;
  display: flex; align-items: center; gap: 2px;
  letter-spacing: 0.04em;
  user-select: none;
}
.brand-kk {
  background: linear-gradient(135deg, #a78bfa 0%, #60a5fa 60%, #38bdf8 100%);
  background-size: 200% auto;
  -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent;
  animation: shimmerText 4s linear infinite;
  text-shadow: none;
  filter: drop-shadow(0 0 8px rgba(167,139,250,0.5));
}
.brand-dot {
  color: rgba(255,255,255,0.3);
  font-weight: 300;
  margin: 0 1px;
  -webkit-text-fill-color: rgba(255,255,255,0.3);
}
.brand-proxy {
  background: linear-gradient(135deg, #38bdf8 0%, #818cf8 60%, #c084fc 100%);
  background-size: 200% auto;
  -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent;
  animation: shimmerText 4s linear infinite reverse;
}
[data-theme="light"] .brand-kk {
  background: linear-gradient(135deg, #6366f1 0%, #3b82f6 50%, #0891b2 100%);
  background-size: 200% auto;
  -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent;
  filter: drop-shadow(0 0 6px rgba(99,102,241,0.3));
}
[data-theme="light"] .brand-dot { color: rgba(0,0,0,0.2); -webkit-text-fill-color: rgba(0,0,0,0.2); }
[data-theme="light"] .brand-proxy {
  background: linear-gradient(135deg, #0891b2 0%, #6366f1 60%, #7c3aed 100%);
  background-size: 200% auto;
  -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent;
}
@keyframes shimmerText {
  0%   { background-position: 0% center; }
  50%  { background-position: 100% center; }
  100% { background-position: 0% center; }
}
.sc-icon { width: 12px; height: 12px; flex-shrink: 0; opacity: 0.8; }
.sc-label { font-size: 10px; opacity: 0.75; }
.copy-btn { display: inline-flex; align-items: center; gap: 4px; }
.stats-pills { display: flex; gap: 5px; align-items: center; }
.sc {
  padding: 3px 10px;
  background: rgba(255,255,255,0.06);
  border: 1px solid rgba(255,255,255,0.09);
  border-radius: 20px; font-size: 11px; color: var(--text-muted);
  display: flex; align-items: center; gap: 4px;
  transition: all .2s;
}
.sc:hover {
  background: rgba(255,255,255,0.10);
  border-color: rgba(255,255,255,0.15);
  transform: translateY(-1px);
}
.sc b { font-family: var(--mono); color: var(--text); font-weight: 600; margin: 0 1px; }
.sc-ok { color: var(--green); }
.sc-ok b { color: var(--green); }
.sc-deg { color: var(--orange); }
.sc-deg b { color: var(--orange); }
.sc-err { color: var(--red); }
.sc-err b { color: var(--red); }
.header-right { display: flex; align-items: center; gap: 7px; justify-content: flex-end; }
.hdr-btn {
  padding: 5px 12px; font-size: 11px; font-weight: 500;
  background: rgba(255,255,255,0.06);
  border: 1px solid rgba(255,255,255,0.10);
  border-radius: var(--radius-sm); color: var(--t2);
  cursor: pointer; transition: all .2s;
  letter-spacing: 0.01em;
  position: relative; overflow: hidden;
}
.hdr-btn::before {
  content: '';
  position: absolute; inset: 0;
  background: linear-gradient(135deg, rgba(255,255,255,0.08) 0%, transparent 100%);
  opacity: 0; transition: opacity .2s;
  border-radius: inherit;
}
.hdr-btn:hover::before { opacity: 1; }
.hdr-btn:hover {
  border-color: rgba(255,255,255,0.2);
  color: var(--t1);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.2);
}
.hdr-btn:active { transform: translateY(0); box-shadow: none; }
[data-theme="light"] .hdr-btn {
  background: rgba(255,255,255,0.75);
  border-color: rgba(226,232,240,0.9);
  color: var(--t2);
  box-shadow: 0 1px 3px rgba(0,0,0,.05);
}
[data-theme="light"] .hdr-btn:hover {
  background: rgba(255,255,255,0.95);
  border-color: rgba(148,163,184,0.7);
  color: var(--t1);
  box-shadow: 0 4px 12px rgba(0,0,0,.08);
}
.clear-btn:hover { border-color: color-mix(in srgb, var(--red) 50%, transparent) !important; color: var(--red) !important; }
.theme-btn:hover { border-color: color-mix(in srgb, var(--purple) 50%, transparent) !important; color: var(--purple) !important; }
.logout-btn:hover { border-color: color-mix(in srgb, var(--orange) 50%, transparent) !important; color: var(--orange) !important; }
.copy-btn:hover { border-color: color-mix(in srgb, var(--blue) 50%, transparent) !important; color: var(--blue) !important; }
.config-btn { display: inline-flex; align-items: center; gap: 4px; }
.config-btn svg { flex-shrink: 0; transition: transform .4s; }
.config-btn:hover svg { transform: rotate(60deg); }
.config-btn:hover { border-color: color-mix(in srgb, var(--accent) 50%, transparent) !important; color: var(--accent) !important; }

.conn {
  display: flex; align-items: center; gap: 5px;
  font-size: 10px; font-weight: 500;
  padding: 4px 10px; border-radius: 20px;
  border: 1px solid transparent;
  background: rgba(255,255,255,0.06);
  transition: all .2s;
}
[data-theme="light"] .conn {
  background: rgba(255,255,255,0.85);
  border-color: rgba(226,232,240,0.9);
}
.conn.on {
  color: var(--green);
  border-color: color-mix(in srgb, var(--green) 30%, transparent);
  background: color-mix(in srgb, var(--green) 8%, transparent);
}
.conn.off {
  color: var(--red);
  border-color: color-mix(in srgb, var(--red) 30%, transparent);
  background: color-mix(in srgb, var(--red) 8%, transparent);
}
.conn .d { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
.conn.on .d {
  background: var(--green);
  box-shadow: 0 0 6px var(--green);
  animation: pulse 2s infinite;
}
.conn.off .d { background: var(--red); }
@keyframes pulse {
  0%,100% { opacity: 1; box-shadow: 0 0 6px var(--green); }
  50% { opacity: .4; box-shadow: 0 0 2px var(--green); }
}

/* ===== 手机端响应式 ===== */
@media (max-width: 767px) {
  .app-header {
    grid-template-columns: 1fr auto;
    padding: 10px 12px;
    gap: 8px;
  }
  .header-center { display: none; }
  h1 { font-size: 13px; }
  .header-right { gap: 5px; }
  .copy-btn, .clear-btn, .logout-btn { display: none; }
  .hdr-btn { padding: 5px 9px; font-size: 11px; }
  .conn { padding: 4px 8px; }
}
</style>
