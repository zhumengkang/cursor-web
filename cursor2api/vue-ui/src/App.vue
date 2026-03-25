<template>
  <div class="app">
    <!-- 粒子画布背景 -->
    <canvas ref="particleCanvas" class="particle-canvas" />
    <template v-if="authChecked">
      <LoginPage v-if="!isLoggedIn" @loggedIn="onLogin" />
      <template v-else>
        <AppHeader :connected="sseConnected" @openConfig="configDrawerVisible = true" />
        <div class="tab-bar">
          <button :class="['tab-btn', activeTab === 'logs' ? 'active' : '']" @click="activeTab = 'logs'">请求日志</button>
          <button :class="['tab-btn', activeTab === 'keys' ? 'active' : '']" @click="activeTab = 'keys'">API Keys</button>
          <button :class="['tab-btn', activeTab === 'nodes' ? 'active' : '']" @click="activeTab = 'nodes'">节点管理</button>
        </div>
        <div class="main" :class="{ 'show-detail': showDetail }">
          <template v-if="activeTab === 'logs'">
            <RequestList />
            <DetailPanel />
          </template>
          <KeysPanel v-else-if="activeTab === 'keys'" />
          <NodesPanel v-else-if="activeTab === 'nodes'" />
        </div>
        <ConfigDrawer :visible="configDrawerVisible" @close="configDrawerVisible = false" />
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, provide, onMounted, onUnmounted, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useAuthStore } from './stores/auth';
import { useLogsStore } from './stores/logs';
import { useStatsStore } from './stores/stats';
import { useSSE } from './composables/useSSE';
import LoginPage from './components/LoginPage.vue';
import AppHeader from './components/AppHeader.vue';
import RequestList from './components/RequestList.vue';
import DetailPanel from './components/DetailPanel.vue';
import ConfigDrawer from './components/ConfigDrawer.vue';
import KeysPanel from './components/KeysPanel.vue';
import NodesPanel from './components/NodesPanel.vue';

const auth = useAuthStore();
const logsStore = useLogsStore();
const statsStore = useStatsStore();
const { loggedIn: isLoggedIn } = storeToRefs(auth);

const authChecked = ref(false);
const sseConnected = ref(false);
const configDrawerVisible = ref(false);
const activeTab = ref<'logs' | 'keys' | 'nodes'>('logs');

// 手机端：控制显示列表还是详情
const showDetail = ref(false);
provide('showDetail', showDetail);

// 初始化主题（避免闪烁）
const savedTheme = localStorage.getItem('cursor2api_theme') ?? 'light';
document.documentElement.setAttribute('data-theme', savedTheme);

const { connect: connectSSE, disconnect: disconnectSSE } = useSSE((connected) => { sseConnected.value = connected; });

onMounted(async () => {
  // URL 参数 token 优先：?token=sk-xxx
  const urlToken = new URLSearchParams(location.search).get('token');
  if (urlToken) {
    auth.setToken(urlToken);
    // 清除 URL 参数，避免 token 暴露在浏览器历史
    history.replaceState(null, '', location.pathname);
  }
  try {
    const res = await fetch('/api/vue/stats', {
      headers: auth.token ? { Authorization: `Bearer ${auth.token}` } : {},
    });
    if (res.ok) {
      // 服务端不需要授权 或 token 有效，直接进主页
      auth.loggedIn = true;
    } else if (res.status === 401) {
      // 需要授权，检查本地 token
      auth.loggedIn = auth.isLoggedIn();
    } else {
      auth.loggedIn = auth.isLoggedIn();
    }
  } catch {
    auth.loggedIn = auth.isLoggedIn();
  }
  authChecked.value = true;
  if (isLoggedIn.value) {
    await Promise.all([logsStore.loadRequests(), statsStore.load()]);
    connectSSE();
  }
});

// 退出登录时断开 SSE，仅清空前端状态
watch(isLoggedIn, (val) => {
  if (!val) {
    disconnectSSE();
    logsStore.resetState();
  }
});

async function onLogin() {
  await Promise.all([logsStore.loadRequests(), statsStore.load()]);
  connectSSE();
}

// ===== 粒子系统 =====
const particleCanvas = ref<HTMLCanvasElement | null>(null);
let animFrame = 0;

function initParticles() {
  const canvas = particleCanvas.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d')!;

  function resize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
  }
  resize();
  window.addEventListener('resize', resize);

  const isDark = () => document.documentElement.getAttribute('data-theme') !== 'light';

  // 粒子定义
  interface Particle {
    x: number; y: number;
    vx: number; vy: number;
    r: number; alpha: number;
    color: string;
    twinkle: number;
  }

  const COLORS_DARK  = ['#4db8ff','#c084fc','#00ffe0','#ff6eb4','#00ff9f','#ffe066','#ff4d6a','#818cf8'];
  const COLORS_LIGHT = ['#3b82f6','#8b5cf6','#059669','#ec4899','#0891b2','#6366f1'];

  const N = 160;
  const particles: Particle[] = [];

  function mkParticle(): Particle {
    const dark = isDark();
    const colors = dark ? COLORS_DARK : COLORS_LIGHT;
    return {
      x: Math.random() * canvas.width,
      y: Math.random() * canvas.height,
      vx: (Math.random() - 0.5) * 0.5,
      vy: (Math.random() - 0.5) * 0.5,
      r: dark ? Math.random() * 2.2 + 0.5 : Math.random() * 1.8 + 0.4,
      alpha: dark ? Math.random() * 0.8 + 0.3 : Math.random() * 0.6 + 0.2,
      color: colors[Math.floor(Math.random() * colors.length)],
      twinkle: Math.random() * Math.PI * 2,
    };
  }

  for (let i = 0; i < N; i++) particles.push(mkParticle());

  // 鼠标位置
  let mx = -9999, my = -9999;
  const onMove = (e: MouseEvent) => { mx = e.clientX; my = e.clientY; };
  window.addEventListener('mousemove', onMove);

  const LINK_DIST = 120;
  const MOUSE_DIST = 160;

  function draw() {
    animFrame = requestAnimationFrame(draw);
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const dark = isDark();
    const t = performance.now() / 1000;

    // 更新 + 绘制粒子
    for (const p of particles) {
      p.x += p.vx;
      p.y += p.vy;
      p.twinkle += 0.02;

      // 边界回绕
      if (p.x < -10) p.x = canvas.width + 10;
      if (p.x > canvas.width + 10) p.x = -10;
      if (p.y < -10) p.y = canvas.height + 10;
      if (p.y > canvas.height + 10) p.y = -10;

      // 鼠标斥力
      const dx = p.x - mx, dy = p.y - my;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < MOUSE_DIST && dist > 0) {
        const force = (MOUSE_DIST - dist) / MOUSE_DIST * 0.8;
        p.vx += (dx / dist) * force * 0.15;
        p.vy += (dy / dist) * force * 0.15;
        // 速度限制
        const spd = Math.sqrt(p.vx * p.vx + p.vy * p.vy);
        if (spd > 2.5) { p.vx = p.vx / spd * 2.5; p.vy = p.vy / spd * 2.5; }
      } else {
        // 阻尼回归
        p.vx *= 0.99;
        p.vy *= 0.99;
      }

      const a = p.alpha * (0.7 + 0.3 * Math.sin(p.twinkle));
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
      ctx.fillStyle = p.color + Math.floor(a * 255).toString(16).padStart(2, '0');
      ctx.fill();

      // 发光晕（暗色模式更强烈）
      if (dark) {
        const glowR = p.r * 5;
        ctx.beginPath();
        ctx.arc(p.x, p.y, glowR, 0, Math.PI * 2);
        const g = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, glowR);
        g.addColorStop(0, p.color + '55');
        g.addColorStop(0.4, p.color + '22');
        g.addColorStop(1, p.color + '00');
        ctx.fillStyle = g;
        ctx.fill();
      }
    }

    // 连线
    for (let i = 0; i < particles.length; i++) {
      for (let j = i + 1; j < particles.length; j++) {
        const a = particles[i], b = particles[j];
        const dx = a.x - b.x, dy = a.y - b.y;
        const d = Math.sqrt(dx * dx + dy * dy);
        if (d < LINK_DIST) {
          const alpha = (1 - d / LINK_DIST) * (dark ? 0.28 : 0.10);
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          if (dark) {
            // 用粒子自身颜色混合连线
            const grad = ctx.createLinearGradient(a.x, a.y, b.x, b.y);
            grad.addColorStop(0, a.color + Math.floor(alpha * 255).toString(16).padStart(2,'0'));
            grad.addColorStop(1, b.color + Math.floor(alpha * 255).toString(16).padStart(2,'0'));
            ctx.strokeStyle = grad;
            ctx.lineWidth = 0.8;
          } else {
            ctx.strokeStyle = `rgba(99,102,241,${alpha})`;
            ctx.lineWidth = 0.6;
          }
          ctx.stroke();
        }
      }
    }

    // 鼠标连线（连接最近5个粒子）
    if (mx > 0) {
      const near = particles
        .map(p => ({ p, d: Math.hypot(p.x - mx, p.y - my) }))
        .filter(x => x.d < MOUSE_DIST)
        .sort((a, b) => a.d - b.d)
        .slice(0, 5);
      for (const { p, d } of near) {
        const alpha = (1 - d / MOUSE_DIST) * (dark ? 0.7 : 0.3);
        ctx.beginPath();
        ctx.moveTo(mx, my);
        ctx.lineTo(p.x, p.y);
        if (dark) {
          const grad = ctx.createLinearGradient(mx, my, p.x, p.y);
          grad.addColorStop(0, `rgba(77,184,255,${alpha})`);
          grad.addColorStop(1, p.color + Math.floor(alpha * 0.6 * 255).toString(16).padStart(2,'0'));
          ctx.strokeStyle = grad;
          ctx.lineWidth = 1.2;
          // 鼠标光晕
          ctx.shadowColor = '#4db8ff';
          ctx.shadowBlur = 6;
        } else {
          ctx.strokeStyle = `rgba(59,130,246,${alpha})`;
          ctx.lineWidth = 0.8;
          ctx.shadowBlur = 0;
        }
        ctx.stroke();
        ctx.shadowBlur = 0;
      }
    }
  }

  draw();

  // 清理函数
  return () => {
    cancelAnimationFrame(animFrame);
    window.removeEventListener('resize', resize);
    window.removeEventListener('mousemove', onMove);
  };
}

let cleanupParticles: (() => void) | undefined;
onMounted(() => { cleanupParticles = initParticles() ?? undefined; });
onUnmounted(() => cleanupParticles?.());
</script>

<style>
@import 'highlight.js/styles/github-dark.css';

/* ===== Canvas 粒子背景 ===== */
.particle-canvas {
  position: fixed;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0;
}
.app {
  position: relative;
  z-index: 1;
}

/* ===== Light Theme ===== */
:root, [data-theme="light"] {
  --bg0: #f0f4f8;
  --bg1: #ffffff;
  --bg2: #f7f9fc;
  --bg3: #edf2f7;
  --bg-card: #ffffff;
  --bdr: #e2e8f0;
  --bdr2: #cbd5e1;
  --t1: #1e293b;
  --t2: #475569;
  --t3: #94a3b8;
  --blue: #3b82f6;
  --cyan: #0891b2;
  --green: #059669;
  --yellow: #d97706;
  --red: #dc2626;
  --purple: #7c3aed;
  --pink: #db2777;
  --orange: #ea580c;
  --mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', 'SF Mono', 'Consolas', 'Menlo', monospace;
  --sans: -apple-system, BlinkMacSystemFont, 'PingFang SC', 'Microsoft YaHei', 'Segoe UI', sans-serif;
  --shadow-sm: 0 1px 2px rgba(0,0,0,.05);
  --shadow: 0 1px 3px rgba(0,0,0,.08), 0 1px 2px rgba(0,0,0,.04);
  --shadow-md: 0 4px 6px rgba(0,0,0,.06), 0 2px 4px rgba(0,0,0,.04);
  --shadow-lg: 0 10px 30px rgba(0,0,0,.08), 0 4px 12px rgba(0,0,0,.04);
  --radius: 12px;
  --radius-sm: 8px;
  --radius-lg: 16px;
  /* aliases */
  --bg: var(--bg0);
  --card-bg: var(--bg1);
  --border: var(--bdr);
  --border-faint: #f1f5f9;
  --text: var(--t1);
  --text-muted: var(--t2);
  --text-dim: var(--t3);
  --accent: var(--blue);
  --accent2: #6366f1;
  --pill-bg: var(--bg3);
  --hover-bg: var(--bg3);
  --active-bg: #eff6ff;
  /* 玻璃态 */
  --glass-bg: rgba(255,255,255,0.72);
  --glass-border: rgba(255,255,255,0.9);
  --glass-blur: blur(16px);
  --glass-shadow: 0 8px 32px rgba(59,130,246,.10), 0 1px 2px rgba(0,0,0,.05);
  /* 渐变 accent */
  --gradient-accent: linear-gradient(135deg, #3b82f6 0%, #6366f1 100%);
  --gradient-green: linear-gradient(135deg, #059669 0%, #0891b2 100%);
  --gradient-warm: linear-gradient(135deg, #f59e0b 0%, #ef4444 100%);
}

/* ===== Dark Theme (赛博朋克霓虹) ===== */
[data-theme="dark"] {
  --bg0: #03060f;
  --bg1: #070d1a;
  --bg2: #0c1426;
  --bg3: #111c30;
  --bg-card: #070d1a;
  --bdr: #1a2a45;
  --bdr2: #1e3560;
  --t1: #e8f0ff;
  --t2: #7a90b8;
  --t3: #3a4d6a;
  --blue: #4db8ff;
  --cyan: #00ffe0;
  --green: #00ff9f;
  --yellow: #ffe066;
  --red: #ff4d6a;
  --purple: #c084fc;
  --pink: #ff6eb4;
  --orange: #ff9d4d;
  --mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', 'SF Mono', 'Consolas', 'Menlo', monospace;
  --sans: -apple-system, BlinkMacSystemFont, 'PingFang SC', 'Microsoft YaHei', 'Segoe UI', sans-serif;
  --shadow-sm: 0 0 8px rgba(77,184,255,.15);
  --shadow: 0 0 16px rgba(77,184,255,.12), 0 2px 8px rgba(0,0,0,.6);
  --shadow-md: 0 0 24px rgba(77,184,255,.15), 0 4px 16px rgba(0,0,0,.6);
  --shadow-lg: 0 0 40px rgba(77,184,255,.2), 0 8px 32px rgba(0,0,0,.7);
  --radius: 12px;
  --radius-sm: 8px;
  --radius-lg: 16px;
  --bg: var(--bg0);
  --card-bg: var(--bg1);
  --border: rgba(77,184,255,0.15);
  --border-faint: rgba(77,184,255,0.06);
  --text: var(--t1);
  --text-muted: var(--t2);
  --text-dim: var(--t3);
  --accent: var(--blue);
  --accent2: var(--purple);
  --pill-bg: rgba(77,184,255,0.08);
  --hover-bg: rgba(77,184,255,0.06);
  --active-bg: rgba(77,184,255,0.12);
  /* 霓虹玻璃态 */
  --glass-bg: rgba(7,13,26,0.80);
  --glass-border: rgba(77,184,255,0.18);
  --glass-blur: blur(20px) saturate(180%);
  --glass-shadow: 0 0 30px rgba(77,184,255,.12), 0 8px 32px rgba(0,0,0,.5);
  /* 霓虹渐变 */
  --gradient-accent: linear-gradient(135deg, #4db8ff 0%, #c084fc 100%);
  --gradient-green: linear-gradient(135deg, #00ff9f 0%, #00ffe0 100%);
  --gradient-warm: linear-gradient(135deg, #ffe066 0%, #ff4d6a 100%);
  /* 霓虹发光 */
  --neon-blue: 0 0 8px #4db8ff, 0 0 20px rgba(77,184,255,.4);
  --neon-cyan: 0 0 8px #00ffe0, 0 0 20px rgba(0,255,224,.4);
  --neon-green: 0 0 8px #00ff9f, 0 0 20px rgba(0,255,159,.4);
  --neon-purple: 0 0 8px #c084fc, 0 0 20px rgba(192,132,252,.4);
}

/* highlight.js 亮色主题覆盖 */
[data-theme="light"] .hljs { background: #f6f8fa; color: #24292e; }
[data-theme="light"] .hljs-comment, [data-theme="light"] .hljs-quote { color: #6a737d; }
[data-theme="light"] .hljs-keyword, [data-theme="light"] .hljs-selector-tag { color: #d73a49; font-weight: bold; }
[data-theme="light"] .hljs-string, [data-theme="light"] .hljs-attr { color: #032f62; }
[data-theme="light"] .hljs-number, [data-theme="light"] .hljs-literal { color: #005cc5; }
[data-theme="light"] .hljs-title, [data-theme="light"] .hljs-section { color: #6f42c1; font-weight: bold; }
[data-theme="light"] .hljs-built_in, [data-theme="light"] .hljs-type { color: #e36209; }
[data-theme="light"] .hljs-variable, [data-theme="light"] .hljs-name { color: #24292e; }

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: var(--sans);
  font-size: 14px;
  background: var(--bg0);
  color: var(--t1);
  height: 100vh;
  overflow: hidden;
}

/* ===== 动态背景 ===== */
[data-theme="light"] body {
  background:
    radial-gradient(ellipse 70% 50% at 15% -5%, rgba(99,102,241,0.08) 0%, transparent 55%),
    radial-gradient(ellipse 60% 40% at 90% 20%, rgba(59,130,246,0.07) 0%, transparent 50%),
    radial-gradient(ellipse 50% 60% at 50% 110%, rgba(139,92,246,0.05) 0%, transparent 55%),
    linear-gradient(160deg, #eef2ff 0%, #f0f4f8 35%, #f8faff 65%, #edf2f7 100%);
  background-attachment: fixed;
}

[data-theme="dark"] body {
  background:
    linear-gradient(rgba(77,184,255,0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(77,184,255,0.03) 1px, transparent 1px),
    linear-gradient(rgba(77,184,255,0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(77,184,255,0.015) 1px, transparent 1px),
    #03060f;
  background-size: 100px 100px, 100px 100px, 20px 20px, 20px 20px;
  background-attachment: fixed;
}

[data-theme="dark"] body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(ellipse 80% 40% at 20% 0%, rgba(77,184,255,0.07) 0%, transparent 60%),
    radial-gradient(ellipse 60% 30% at 80% 100%, rgba(192,132,252,0.06) 0%, transparent 55%);
  z-index: 0;
}

/* ===== 浮动光晕动画 ===== */
[data-theme="dark"] body::before {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(circle 600px at var(--mx, 50%) var(--my, 30%), rgba(96,165,250,0.04) 0%, transparent 70%);
  z-index: 0;
  transition: background 0.5s;
}

.app {
  display: flex; flex-direction: column;
  height: 100vh; color: var(--t1);
  position: relative; z-index: 1;
}

/* ===== 顶部 Tab 栏 ===== */
.tab-bar {
  display: flex;
  gap: 2px;
  padding: 0 16px;
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  border-bottom: 1px solid var(--glass-border);
  flex-shrink: 0;
  position: relative;
  z-index: 10;
}
.tab-btn {
  padding: 10px 22px;
  font-size: 14px;
  font-weight: 600;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--t2);
  cursor: pointer;
  transition: color .2s, border-color .2s;
  margin-bottom: -1px;
  position: relative;
  letter-spacing: 0.02em;
}
.tab-btn::after {
  content: '';
  position: absolute;
  left: 50%; bottom: -1px;
  width: 0; height: 2px;
  background: var(--gradient-accent);
  border-radius: 2px 2px 0 0;
  transition: width .25s, left .25s;
}
.tab-btn:hover { color: var(--t1); }
.tab-btn.active { color: var(--accent); border-bottom-color: transparent; }
.tab-btn.active::after { width: 100%; left: 0; }

.main { display: flex; flex: 1; overflow: hidden; }

/* ===== 全局过渡动画 ===== */
@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}
@keyframes fadeIn {
  from { opacity: 0; }
  to   { opacity: 1; }
}
@keyframes slideInRight {
  from { opacity: 0; transform: translateX(20px); }
  to   { opacity: 1; transform: translateX(0); }
}
@keyframes shimmer {
  0%   { background-position: -200% center; }
  100% { background-position: 200% center; }
}

/* ===== 响应式：手机端 ===== */
@media (max-width: 767px) {
  body { overflow: hidden; }
  .main {
    position: relative;
    overflow: hidden;
  }
  .main.show-detail .detail-panel {
    transform: translateX(0) !important;
  }
  .tab-btn { padding: 8px 14px; font-size: 13px; }
}

/* ===== 滚动条美化 ===== */
::-webkit-scrollbar { width: 5px; height: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb {
  background: var(--bdr2);
  border-radius: 10px;
}
::-webkit-scrollbar-thumb:hover { background: var(--t3); }

/* ===== 全局卡片/面板增强 ===== */
.glass-card {
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius);
  box-shadow: var(--glass-shadow);
}

/* ===== 全局按钮聚焦环 ===== */
button:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 4px;
}

/* ===== 渐变文字 helper ===== */
.grad-text {
  background: var(--gradient-accent);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* ===== 暗色模式：霓虹赛博朋克覆盖 ===== */
[data-theme="dark"] .tab-bar {
  background: rgba(7,13,26,0.85);
  border-bottom: 1px solid rgba(77,184,255,0.2);
  box-shadow: 0 1px 0 rgba(77,184,255,0.08), 0 4px 20px rgba(0,0,0,0.4);
}
[data-theme="dark"] .tab-btn.active {
  color: #4db8ff;
  text-shadow: 0 0 10px rgba(77,184,255,0.8);
}
[data-theme="dark"] .tab-btn.active::after {
  background: linear-gradient(90deg, #4db8ff, #c084fc);
  box-shadow: 0 0 8px #4db8ff, 0 0 16px rgba(77,184,255,0.4);
}
[data-theme="dark"] .request-list {
  background: rgba(7,13,26,0.7);
  border-right: 1px solid rgba(77,184,255,0.12);
}
[data-theme="dark"] .detail-panel,
[data-theme="dark"] .app-header {
  background: rgba(7,13,26,0.85);
}
[data-theme="dark"] .app-header {
  border-bottom: 1px solid rgba(77,184,255,0.2);
  box-shadow: 0 1px 0 rgba(77,184,255,0.08), 0 4px 24px rgba(0,0,0,0.5);
}
[data-theme="dark"] .ri {
  border-color: rgba(77,184,255,0.08);
  background: rgba(7,13,26,0.5);
}
[data-theme="dark"] .ri:hover {
  border-color: rgba(77,184,255,0.22);
  background: rgba(77,184,255,0.05);
  box-shadow: 0 0 12px rgba(77,184,255,0.08), inset 0 0 12px rgba(77,184,255,0.02);
}
[data-theme="dark"] .ri.sel {
  border-color: rgba(77,184,255,0.35);
  border-left-color: #4db8ff;
  background: rgba(77,184,255,0.07);
  box-shadow: 0 0 16px rgba(77,184,255,0.12), inset 0 0 20px rgba(77,184,255,0.04);
}
[data-theme="dark"] .tb.a {
  background: linear-gradient(135deg, rgba(0,255,224,0.15), rgba(77,184,255,0.15));
  border-color: #00ffe0;
  color: #00ffe0;
  box-shadow: 0 0 8px rgba(0,255,224,0.3), inset 0 0 8px rgba(0,255,224,0.05);
  text-shadow: 0 0 8px rgba(0,255,224,0.8);
}
[data-theme="dark"] .fb.a {
  background: linear-gradient(135deg, rgba(77,184,255,0.15), rgba(192,132,252,0.15));
  border-color: #4db8ff;
  color: #4db8ff;
  box-shadow: 0 0 8px rgba(77,184,255,0.3);
  text-shadow: 0 0 8px rgba(77,184,255,0.8);
}
[data-theme="dark"] .sc-ok b { text-shadow: 0 0 8px rgba(0,255,159,0.6); }
[data-theme="dark"] .sc-err b { text-shadow: 0 0 8px rgba(255,77,106,0.6); }
[data-theme="dark"] ::-webkit-scrollbar-thumb {
  background: rgba(77,184,255,0.2);
}
[data-theme="dark"] ::-webkit-scrollbar-thumb:hover {
  background: rgba(77,184,255,0.4);
  box-shadow: 0 0 6px rgba(77,184,255,0.4);
}
</style>
