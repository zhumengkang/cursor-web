<template>
  <div class="keys-panel">
    <!-- Tab 切换 -->
    <div class="tab-bar">
      <button :class="['tab-btn', activeTab === 'keys' ? 'active' : '']" @click="activeTab = 'keys'">API Keys</button>
      <button :class="['tab-btn', activeTab === 'stats' ? 'active' : '']" @click="activeTab = 'stats'; loadStats()">用量统计</button>
    </div>

    <!-- ===== Tab: API Keys 管理 ===== -->
    <div v-if="activeTab === 'keys'" class="tab-content">
      <div class="section">
        <div class="section-hdr">
          <div class="section-title">API Keys</div>
          <button class="btn-add" @click="showAddForm = true">+ 添加</button>
        </div>

        <!-- 添加表单 -->
        <div v-if="showAddForm" class="key-form">
          <input v-model="form.keyValue" class="inp" placeholder="Token 值，例如 sk-xxxxxxxx（必填）" />
          <div class="form-actions">
            <button class="btn-save" @click="onAdd" :disabled="!form.keyValue">保存</button>
            <button class="btn-cancel" @click="cancelAdd">取消</button>
          </div>
          <div v-if="formError" class="form-error">{{ formError }}</div>
        </div>

        <div v-if="loading" class="loading">加载中…</div>
        <div v-else-if="keys.length === 0" class="empty">暂无 API Key</div>
        <div v-else class="keys-list">
          <div v-for="key in keys" :key="key.id" class="key-row">
            <div class="key-main">
              <div class="key-value mono">{{ key.keyValue }}</div>
            </div>
            <div class="key-actions">
              <button class="btn-act btn-copy" @click="onCopy(key.keyValue)" title="复制">复制</button>
              <button class="btn-act btn-del" @click="onDelete(key.id)">删除</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ===== Tab: 用量统计 ===== -->
    <div v-if="activeTab === 'stats'" class="tab-content">
      <!-- 时间范围 + 粒度筛选 + Key 过滤 -->
      <div class="filter-bar">
        <div class="filter-group">
          <button v-for="r in ranges" :key="r.label"
            :class="['filter-btn', activeRange === r.label ? 'active' : '']"
            @click="setRange(r)">{{ r.label }}</button>
        </div>
        <div class="filter-group">
          <button v-for="g in granularities" :key="g.value"
            :class="['filter-btn', granularity === g.value ? 'active' : '']"
            @click="setGranularity(g.value)">{{ g.label }}</button>
        </div>
        <div class="filter-group">
          <input class="key-select" v-model="selectedKey" list="key-list"
            placeholder="全部 Key（可输入或选择）"
            @change="loadStats()" @keyup.enter="loadStats()" />
          <datalist id="key-list">
            <option v-for="k in keys" :key="k.id" :value="k.keyValue" />
          </datalist>
        </div>
      </div>

      <!-- 统计卡片 -->
      <div v-if="stats" class="stats-row">
        <div class="stat-card">
          <div class="stat-val">{{ stats.totalRequests }}</div>
          <div class="stat-lbl">总请求</div>
        </div>
        <div class="stat-card stat-ok">
          <div class="stat-val">{{ stats.successCount }}</div>
          <div class="stat-lbl">成功</div>
        </div>
        <div class="stat-card stat-err">
          <div class="stat-val">{{ stats.errorCount }}</div>
          <div class="stat-lbl">失败</div>
        </div>
        <div class="stat-card">
          <div class="stat-val">{{ fmtTokens(stats.totalTokens) }}</div>
          <div class="stat-lbl">总 Token</div>
        </div>
        <div class="stat-card">
          <div class="stat-val">{{ fmtTokens(stats.totalInputTokens) }}</div>
          <div class="stat-lbl">输入 Token</div>
        </div>
        <div class="stat-card">
          <div class="stat-val">{{ fmtTokens(stats.totalOutputTokens) }}</div>
          <div class="stat-lbl">输出 Token</div>
        </div>
        <div class="stat-card">
          <div class="stat-val">{{ stats.successRate.toFixed(1) }}%</div>
          <div class="stat-lbl">成功率</div>
        </div>
        <div class="stat-card" v-if="stats.avgResponseTime">
          <div class="stat-val">{{ fmtMs(stats.avgResponseTime) }}</div>
          <div class="stat-lbl">平均响应</div>
        </div>
      </div>
      <div v-else-if="statsLoading" class="loading">加载中…</div>
      <div v-else class="empty">暂无统计数据</div>

      <!-- 按模型分布 -->
      <div v-if="stats?.modelsBreakdown?.length" class="section">
        <div class="section-title">模型分布</div>
        <table class="breakdown-table">
          <thead><tr><th>模型</th><th>请求数</th><th>输入 Token</th><th>输出 Token</th></tr></thead>
          <tbody>
            <tr v-for="m in stats.modelsBreakdown" :key="m.model">
              <td class="model-name">{{ m.model }}</td>
              <td>{{ m.count }}</td>
              <td>{{ fmtTokens(m.inputTokens) }}</td>
              <td>{{ fmtTokens(m.outputTokens) }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 时间趋势 -->
      <div v-if="stats?.periodBreakdown?.length" class="section">
        <div class="section-title">用量趋势</div>
        <table class="breakdown-table">
          <thead><tr><th>时间</th><th>请求数</th><th>输入 Token</th><th>输出 Token</th></tr></thead>
          <tbody>
            <tr v-for="d in stats.periodBreakdown" :key="d.date">
              <td>{{ d.date }}</td>
              <td>{{ d.count }}</td>
              <td>{{ fmtTokens(d.inputTokens) }}</td>
              <td>{{ fmtTokens(d.outputTokens) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { fetchKeys, createKey, deleteKey, fetchKeyStats } from '../api';
import type { ApiKey, KeyStats } from '../types';

const activeTab = ref<'keys' | 'stats'>('keys');

const keys = ref<ApiKey[]>([]);
const stats = ref<KeyStats | null>(null);
const loading = ref(false);
const statsLoading = ref(false);
const showAddForm = ref(false);
const formError = ref('');
const selectedKey = ref('');

// 时间筛选
type Granularity = 'hour' | 'day' | 'week' | 'month';
const granularity = ref<Granularity>('day');
const activeRange = ref('全部');
const since = ref<number | undefined>(undefined);
const until = ref<number | undefined>(undefined);

const granularities = [
  { label: '小时', value: 'hour' as Granularity },
  { label: '天', value: 'day' as Granularity },
  { label: '周', value: 'week' as Granularity },
  { label: '月', value: 'month' as Granularity },
];

const ranges = [
  { label: '全部', since: undefined as number | undefined },
  { label: '今天', since: () => { const d = new Date(); d.setHours(0,0,0,0); return d.getTime(); } },
  { label: '近7天', since: () => Date.now() - 7 * 86400_000 },
  { label: '近30天', since: () => Date.now() - 30 * 86400_000 },
  { label: '本月', since: () => { const d = new Date(); d.setDate(1); d.setHours(0,0,0,0); return d.getTime(); } },
];

function setRange(r: typeof ranges[0]) {
  activeRange.value = r.label;
  since.value = typeof r.since === 'function' ? r.since() : r.since;
  until.value = undefined;
  loadStats();
}

function setGranularity(g: Granularity) {
  granularity.value = g;
  loadStats();
}

async function loadStats() {
  statsLoading.value = true;
  try {
    stats.value = await fetchKeyStats({ since: since.value, until: until.value, granularity: granularity.value, key: selectedKey.value || undefined });
  } catch (e) {
    stats.value = null;
  } finally {
    statsLoading.value = false;
  }
}

const form = ref({ keyValue: '' });

function fmtTokens(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
  if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
  return String(n);
}

function fmtMs(ms: number): string {
  if (ms >= 1000) return (ms / 1000).toFixed(1) + 's';
  return Math.round(ms) + 'ms';
}

async function loadKeys() {
  loading.value = true;
  try {
    keys.value = await fetchKeys();
  } finally {
    loading.value = false;
  }
}

function cancelAdd() {
  showAddForm.value = false;
  form.value = { keyValue: '' };
  formError.value = '';
}

async function onAdd() {
  formError.value = '';
  try {
    const created = await createKey({ name: form.value.keyValue, keyValue: form.value.keyValue });
    keys.value.unshift(created);
    cancelAdd();
  } catch (e) {
    formError.value = String(e);
  }
}

async function onDelete(id: string) {
  if (!confirm('确认删除此 Key？')) return;
  try {
    await deleteKey(encodeURIComponent(id));
    keys.value = keys.value.filter(k => k.id !== id);
  } catch (e) {
    alert(String(e));
  }
}

async function onCopy(val: string) {
  await navigator.clipboard.writeText(val);
}

onMounted(loadKeys);
</script>

<style scoped>
.keys-panel { display: flex; flex-direction: column; gap: 16px; padding: 16px; overflow-y: auto; height: 100%; width: 100%; }

/* Tab 切换 */
.tab-bar {
  display: flex; gap: 4px;
  border-bottom: 1px solid var(--border);
  padding-bottom: 0;
  flex-shrink: 0;
}
.tab-btn {
  padding: 7px 18px; font-size: 13px; font-weight: 500;
  background: none; border: none; border-bottom: 2px solid transparent;
  color: var(--t3); cursor: pointer; margin-bottom: -1px;
  transition: color .2s, border-color .2s, background .2s;
  border-radius: 6px 6px 0 0;
}
.tab-btn:hover { color: var(--t1); background: rgba(255,255,255,0.04); }
.tab-btn.active {
  color: var(--accent); border-bottom-color: var(--accent);
  background: linear-gradient(180deg, transparent 50%, color-mix(in srgb, var(--accent) 6%, transparent) 100%);
}

.tab-content { display: flex; flex-direction: column; gap: 16px; }

@media (max-width: 767px) {
  .stats-row { gap: 8px; }
  .stat-card { min-width: calc(50% - 4px); flex: 1 1 calc(50% - 4px); }
  .filter-bar { gap: 8px; }
  .keys-panel { padding: 12px; }
}

.stats-row { display: flex; flex-wrap: wrap; gap: 10px; }
.stat-card {
  background: var(--bg1); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 10px 16px;
  min-width: 90px; text-align: center;
}
.stat-card.stat-ok { border-color: var(--green); }
.stat-card.stat-err { border-color: var(--red); }
.stat-val { font-size: 22px; font-weight: 700; color: var(--t1); }
.stat-lbl { font-size: 13px; color: var(--t3); margin-top: 2px; }

.section { background: var(--bg1); border: 1px solid var(--border); border-radius: var(--radius); padding: 14px; }
.section-hdr { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--t1); margin-bottom: 10px; }
.section-hdr .section-title { margin-bottom: 0; }

.filter-bar { display: flex; gap: 12px; flex-wrap: wrap; }
.filter-group { display: flex; gap: 4px; }
.filter-btn {
  padding: 5px 12px; font-size: 13px; font-weight: 500;
  background: var(--bg2); border: 1px solid var(--border);
  border-radius: 12px; color: var(--t2); cursor: pointer; transition: all .15s;
}
.filter-btn:hover { border-color: var(--accent); color: var(--accent); }
.filter-btn.active { background: var(--accent); border-color: var(--accent); color: #fff; }
.key-select {
  padding: 4px 10px; font-size: 13px;
  background: var(--bg0); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--t2);
  width: 220px;
}
.key-select:focus { border-color: var(--accent); outline: none; }

.breakdown-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.breakdown-table th { text-align: left; color: var(--t3); padding: 6px 8px; border-bottom: 1px solid var(--border); }
.breakdown-table td { padding: 7px 8px; border-bottom: 1px solid var(--border); color: var(--t2); }
.breakdown-table tr:last-child td { border-bottom: none; }
.model-name { font-family: var(--mono); font-size: 11px; color: var(--t1); }

.btn-add {
  padding: 4px 12px; font-size: 12px; font-weight: 500;
  background: var(--accent); color: #fff;
  border: none; border-radius: var(--radius-sm); cursor: pointer;
}
.btn-add:hover { opacity: .85; }

.key-form {
  background: var(--bg2); border: 1px solid var(--border);
  border-radius: var(--radius-sm); padding: 12px;
  display: flex; flex-direction: column; gap: 8px;
  margin-bottom: 12px;
}
.inp {
  width: 100%; padding: 6px 10px; font-size: 12px;
  background: var(--bg0); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--t1); outline: none;
  box-sizing: border-box;
}
.inp:focus { border-color: var(--accent); }
.form-actions { display: flex; gap: 8px; }
.btn-save {
  padding: 5px 14px; font-size: 12px; font-weight: 500;
  background: var(--accent); color: #fff;
  border: none; border-radius: var(--radius-sm); cursor: pointer;
}
.btn-save:disabled { opacity: .5; cursor: not-allowed; }
.btn-cancel {
  padding: 5px 14px; font-size: 12px;
  background: var(--bg2); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--t2); cursor: pointer;
}
.form-error { font-size: 12px; color: var(--red); }

.loading, .empty { color: var(--t3); font-size: 13px; text-align: center; padding: 20px; }

@keyframes key-in {
  from { opacity: 0; transform: translateY(6px); }
  to   { opacity: 1; transform: translateY(0); }
}
.keys-list { display: flex; flex-direction: column; gap: 8px; }
.mono { font-family: var(--mono); font-size: 12px; word-break: break-all; }
.btn-copy { color: var(--blue); border-color: color-mix(in srgb, var(--blue) 50%, transparent); }
.key-row {
  background: rgba(255,255,255,0.03);
  border: 1px solid var(--border);
  border-radius: 10px; padding: 10px 12px;
  display: flex; align-items: flex-start; gap: 10px;
  transition: background .2s, border-color .2s, box-shadow .2s, transform .2s;
  animation: key-in .25s ease both;
}
.key-row:hover {
  background: rgba(255,255,255,0.06);
  border-color: var(--border);
  box-shadow: var(--shadow-sm);
  transform: translateY(-1px);
}
.key-main { flex: 1; min-width: 0; }
.key-value { font-family: var(--mono); font-size: 11px; color: var(--t3); margin-top: 2px; word-break: break-all; }
.key-actions { display: flex; gap: 6px; flex-shrink: 0; align-self: flex-start; }
.btn-act {
  padding: 3px 10px; font-size: 11px;
  background: rgba(255,255,255,0.04); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--t2); cursor: pointer;
  transition: all .2s;
}
.btn-act:hover { border-color: var(--accent); color: var(--accent); background: color-mix(in srgb, var(--accent) 8%, transparent); }
.btn-del:hover { border-color: var(--red); color: var(--red); background: color-mix(in srgb, var(--red) 8%, transparent); }

/* ===== 暗色霓虹覆盖 ===== */
[data-theme="dark"] .section {
  background: rgba(7,13,26,0.7);
  border-color: rgba(77,184,255,0.12);
  box-shadow: 0 0 20px rgba(77,184,255,0.04), inset 0 0 30px rgba(77,184,255,0.02);
}
[data-theme="dark"] .stat-card {
  background: rgba(7,13,26,0.8);
  border-color: rgba(77,184,255,0.12);
}
[data-theme="dark"] .stat-card.stat-ok {
  border-color: rgba(0,255,159,0.3);
  box-shadow: 0 0 12px rgba(0,255,159,0.08);
}
[data-theme="dark"] .stat-card.stat-err {
  border-color: rgba(255,77,106,0.3);
  box-shadow: 0 0 12px rgba(255,77,106,0.08);
}
[data-theme="dark"] .stat-val { color: #4db8ff; text-shadow: 0 0 10px rgba(77,184,255,0.5); }
[data-theme="dark"] .btn-add {
  background: linear-gradient(135deg, #4db8ff, #c084fc);
  box-shadow: 0 0 12px rgba(77,184,255,0.4);
}
[data-theme="dark"] .btn-add:hover {
  box-shadow: 0 0 20px rgba(77,184,255,0.6);
  filter: brightness(1.1);
}
[data-theme="dark"] .key-row {
  background: rgba(7,13,26,0.6);
  border-color: rgba(77,184,255,0.1);
}
[data-theme="dark"] .key-row:hover {
  background: rgba(77,184,255,0.05);
  border-color: rgba(77,184,255,0.25);
  box-shadow: 0 0 16px rgba(77,184,255,0.1);
}
[data-theme="dark"] .filter-btn.active {
  background: linear-gradient(135deg, rgba(77,184,255,0.2), rgba(192,132,252,0.2));
  border-color: #4db8ff;
  color: #4db8ff;
  box-shadow: 0 0 10px rgba(77,184,255,0.3);
  text-shadow: 0 0 8px rgba(77,184,255,0.8);
}
[data-theme="dark"] .inp {
  background: rgba(7,13,26,0.8);
  border-color: rgba(77,184,255,0.15);
  color: var(--t1);
}
[data-theme="dark"] .inp:focus {
  border-color: #4db8ff;
  box-shadow: 0 0 0 3px rgba(77,184,255,0.12), 0 0 12px rgba(77,184,255,0.15);
}
[data-theme="dark"] .btn-save {
  background: linear-gradient(135deg, #4db8ff, #c084fc);
  box-shadow: 0 0 12px rgba(77,184,255,0.4);
}
[data-theme="dark"] .btn-copy {
  color: #4db8ff;
  border-color: rgba(77,184,255,0.35);
  text-shadow: 0 0 8px rgba(77,184,255,0.6);
}
</style>
