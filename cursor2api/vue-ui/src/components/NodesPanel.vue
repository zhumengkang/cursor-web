<template>
  <div class="nodes-panel">
    <!-- 工具栏 -->
    <div class="toolbar">
      <button class="btn-add" @click="showAddForm = !showAddForm">+ 添加代理</button>
      <button class="btn-tool" @click="testAll" :disabled="testing || proxies.length === 0">全部测试</button>
      <button class="btn-tool btn-warn" @click="pruneFailedProxies" :disabled="testing || failedCount === 0">剔除失败 ({{ failedCount }})</button>
      <button class="btn-tool btn-danger" @click="confirmDeleteAll" :disabled="proxies.length === 0">删除全部</button>
    </div>

    <!-- 添加表单 -->
    <div v-if="showAddForm" class="add-form">
      <textarea v-model="addText" class="inp" rows="4"
        placeholder="每行一个代理地址，支持：&#10;http://host:port&#10;https://user:pass@host:port&#10;socks5://host:port&#10;socks4://user:pass@host:port" />
      <div class="form-actions">
        <button class="btn-save" @click="onBatchAdd" :disabled="!addText.trim()">保存</button>
        <button class="btn-cancel" @click="cancelAdd">取消</button>
      </div>
      <div v-if="formError" class="form-error">{{ formError }}</div>
    </div>

    <div v-if="loading" class="empty">加载中…</div>
    <div v-else-if="proxies.length === 0" class="empty">暂无代理，点击「添加代理」开始</div>

    <!-- 代理列表 -->
    <div v-else class="nodes-list">
      <div v-for="(proxy, idx) in proxies" :key="proxy" class="node-row">
        <div class="node-info">
          <span class="node-index mono">#{{ idx + 1 }}</span>
          <span class="node-token mono">{{ proxy }}</span>
          <span :class="['badge', 'badge-' + getResult(proxy).status]">
            <span v-if="getResult(proxy).status === 'testing'" class="spin">⟳</span>
            <span v-else-if="getResult(proxy).status === 'ok'">✓</span>
            <span v-else-if="getResult(proxy).status === 'fail'">✗</span>
            <span v-else>—</span>
            <span v-if="getResult(proxy).status === 'ok' && getResult(proxy).latency !== null">
              {{ getResult(proxy).latency }}ms
            </span>
            <span v-if="getResult(proxy).status === 'fail' && getResult(proxy).error" class="err-msg">
              {{ truncErr(getResult(proxy).error!) }}
            </span>
          </span>
        </div>
        <div class="node-actions">
          <button class="btn-sm" @click="testOne(proxy)" :disabled="getResult(proxy).status === 'testing'">测试</button>
          <button class="btn-sm btn-del" @click="deleteProxy(proxy)">删除</button>
        </div>
      </div>
    </div>

    <!-- 轮询说明 -->
    <div v-if="proxies.length > 1" class="round-robin-hint">
      ↻ 多代理已启用轮询负载均衡，请求将按顺序轮流使用各代理
    </div>

    <!-- 确认删除全部 -->
    <div v-if="showDeleteAllModal" class="modal-overlay" @click.self="showDeleteAllModal = false">
      <div class="modal">
        <div class="modal-title">确认删除全部代理？</div>
        <div class="modal-body">此操作不可撤销，将清空所有 {{ proxies.length }} 个代理。</div>
        <div class="modal-actions">
          <button class="btn-cancel" @click="showDeleteAllModal = false">取消</button>
          <button class="btn-danger" @click="deleteAll">确认删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useNodesStore } from '../stores/nodes';
import { fetchProxies, saveProxies, testProxy } from '../api';

const nodesStore = useNodesStore();

const proxies = ref<string[]>([]);
const loading = ref(true);
const testing = ref(false);
const showAddForm = ref(false);
const addText = ref('');
const formError = ref('');
const showDeleteAllModal = ref(false);

const failedCount = computed(() =>
  proxies.value.filter(p => nodesStore.getResult(p).status === 'fail').length
);

function getResult(proxy: string) {
  return nodesStore.getResult(proxy);
}

function truncErr(err: string): string {
  return err.length > 40 ? err.slice(0, 40) + '…' : err;
}

async function load() {
  loading.value = true;
  try {
    const res = await fetchProxies();
    proxies.value = res.proxies ?? [];
  } catch {
    proxies.value = [];
  } finally {
    loading.value = false;
  }
}

async function testOne(proxy: string) {
  nodesStore.setResult({ proxy, status: 'testing', latency: null, httpStatus: null, error: null, testedAt: null });
  try {
    const res = await testProxy(proxy);
    nodesStore.setResult({
      proxy,
      status: res.ok ? 'ok' : 'fail',
      latency: res.latency ?? null,
      httpStatus: res.status ?? null,
      error: res.error ?? null,
      testedAt: Date.now(),
    });
  } catch (e: unknown) {
    nodesStore.setResult({
      proxy,
      status: 'fail',
      latency: null,
      httpStatus: null,
      error: e instanceof Error ? e.message : String(e),
      testedAt: Date.now(),
    });
  }
}

async function testAll() {
  if (proxies.value.length === 0) return;
  testing.value = true;
  await Promise.all(proxies.value.map(p => testOne(p)));
  testing.value = false;
}

async function onBatchAdd() {
  formError.value = '';
  const lines = addText.value
    .split('\n')
    .map(l => l.trim())
    .filter(l => l.length > 0);

  const valid = lines.filter(l =>
    /^(https?|socks[45]h?):(\/\/)/i.test(l)
  );
  const invalid = lines.filter(l =>
    !/^(https?|socks[45]h?):(\/\/)/i.test(l)
  );

  if (invalid.length > 0) {
    formError.value = `格式错误（需以 http/https/socks4/socks5 开头）：${invalid.slice(0, 3).join(', ')}`;
    return;
  }
  if (valid.length === 0) return;

  const merged = [...new Set([...proxies.value, ...valid])];
  try {
    const res = await saveProxies(merged);
    proxies.value = res.proxies ?? merged;
    addText.value = '';
    showAddForm.value = false;
  } catch (e: unknown) {
    formError.value = e instanceof Error ? e.message : '保存失败';
  }
}

function cancelAdd() {
  showAddForm.value = false;
  addText.value = '';
  formError.value = '';
}

async function deleteProxy(proxy: string) {
  const updated = proxies.value.filter(p => p !== proxy);
  try {
    const res = await saveProxies(updated);
    proxies.value = res.proxies ?? updated;
    nodesStore.clearResult(proxy);
  } catch { /* ignore */ }
}

function confirmDeleteAll() {
  showDeleteAllModal.value = true;
}

async function deleteAll() {
  showDeleteAllModal.value = false;
  try {
    const res = await saveProxies([]);
    proxies.value = res.proxies ?? [];
    nodesStore.clearAll();
  } catch { /* ignore */ }
}

async function pruneFailedProxies() {
  const surviving = proxies.value.filter(p => nodesStore.getResult(p).status !== 'fail');
  try {
    const res = await saveProxies(surviving);
    proxies.value = res.proxies ?? surviving;
    proxies.value
      .filter(p => nodesStore.getResult(p).status === 'fail')
      .forEach(p => nodesStore.clearResult(p));
  } catch { /* ignore */ }
}

onMounted(load);
</script>

<style scoped>
.nodes-panel {
  padding: 16px;
  max-width: 900px;
  margin: 0 auto;
  width: 100%;
  height: 100%;
  overflow-y: auto;
}
.toolbar {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.btn-add {
  padding: 7px 16px;
  border-radius: 8px;
  border: none;
  background: linear-gradient(135deg, #00ffe0, #4db8ff);
  color: #000;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.btn-tool {
  padding: 7px 16px;
  border-radius: 8px;
  border: 1px solid rgba(128,128,128,0.3);
  background: transparent;
  color: var(--t1, #222);
  font-size: 13px;
  cursor: pointer;
}
.btn-tool:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-warn { border-color: #f59e0b; color: #f59e0b; }
.btn-danger-tool { border-color: #ef4444; color: #ef4444; }
.add-form {
  background: rgba(128,128,128,0.06);
  border: 1px solid rgba(128,128,128,0.15);
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 16px;
}
.inp {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid rgba(128,128,128,0.25);
  border-radius: 8px;
  padding: 10px 12px;
  font-size: 13px;
  font-family: monospace;
  background: var(--bg, #fff);
  color: var(--t1, #222);
  resize: vertical;
}
.inp:focus { outline: none; border-color: #4db8ff; box-shadow: 0 0 0 3px rgba(77,184,255,0.12); }
.form-actions { display: flex; gap: 8px; margin-top: 10px; }
.btn-save {
  padding: 7px 18px;
  border-radius: 8px;
  border: none;
  background: linear-gradient(135deg, #4db8ff, #c084fc);
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.btn-save:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-cancel {
  padding: 7px 16px;
  border-radius: 8px;
  border: 1px solid rgba(128,128,128,0.3);
  background: transparent;
  color: var(--t1, #222);
  font-size: 13px;
  cursor: pointer;
}
.form-error { margin-top: 8px; color: #ef4444; font-size: 12px; }
.empty { text-align: center; color: var(--t2, #888); padding: 40px 0; font-size: 14px; }
.nodes-list { display: flex; flex-direction: column; gap: 8px; }
.node-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border: 1px solid rgba(128,128,128,0.15);
  border-radius: 10px;
  background: rgba(128,128,128,0.03);
  gap: 12px;
}
.node-row:hover { border-color: rgba(128,128,128,0.3); }
.node-info { display: flex; align-items: center; gap: 10px; min-width: 0; flex: 1; }
.node-index { font-size: 12px; color: var(--t2, #888); min-width: 28px; }
.node-token { font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.mono { font-family: monospace; }
.badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 12px;
  white-space: nowrap;
  flex-shrink: 0;
}
.badge-idle { background: rgba(128,128,128,0.1); color: var(--t2, #888); }
.badge-testing { background: rgba(77,184,255,0.12); color: #4db8ff; }
.badge-ok { background: rgba(0,255,159,0.12); color: #00b06a; }
.badge-fail { background: rgba(255,77,106,0.12); color: #ef4444; }
.err-msg { max-width: 180px; overflow: hidden; text-overflow: ellipsis; }
.spin { display: inline-block; animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.node-actions { display: flex; gap: 6px; flex-shrink: 0; }
.btn-sm {
  padding: 5px 12px;
  border-radius: 6px;
  border: 1px solid rgba(128,128,128,0.25);
  background: transparent;
  color: var(--t1, #222);
  font-size: 12px;
  cursor: pointer;
}
.btn-sm:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-del { border-color: rgba(239,68,68,0.4); color: #ef4444; }
.round-robin-hint {
  margin-top: 14px;
  padding: 10px 14px;
  border-radius: 8px;
  background: rgba(77,184,255,0.07);
  border: 1px solid rgba(77,184,255,0.2);
  font-size: 13px;
  color: #4db8ff;
}
.modal-overlay {
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.45);
  display: flex; align-items: center; justify-content: center;
  z-index: 999;
}
.modal {
  background: var(--bg, #fff);
  border: 1px solid rgba(128,128,128,0.2);
  border-radius: 14px;
  padding: 24px 28px;
  min-width: 300px;
  max-width: 400px;
}
.modal-title { font-size: 16px; font-weight: 700; margin-bottom: 10px; }
.modal-body { font-size: 14px; color: var(--t2, #888); margin-bottom: 18px; }
.modal-actions { display: flex; gap: 10px; justify-content: flex-end; }
.btn-danger {
  padding: 7px 16px;
  border-radius: 8px;
  border: none;
  background: #ef4444;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}
/* dark theme */
[data-theme="dark"] .node-row {
  border-color: rgba(77,184,255,0.1);
  background: rgba(7,13,26,0.5);
}
[data-theme="dark"] .node-row:hover { border-color: rgba(77,184,255,0.2); }
[data-theme="dark"] .inp {
  background: rgba(7,13,26,0.8);
  border-color: rgba(77,184,255,0.15);
  color: var(--t1);
}
[data-theme="dark"] .inp:focus { border-color: #4db8ff; box-shadow: 0 0 0 3px rgba(77,184,255,0.12); }
[data-theme="dark"] .modal { background: #0d1829; border-color: rgba(77,184,255,0.2); }
[data-theme="dark"] .btn-save { background: linear-gradient(135deg, #4db8ff, #c084fc); }
[data-theme="dark"] .badge-ok { color: #00ff9f; }
[data-theme="dark"] .round-robin-hint { background: rgba(77,184,255,0.05); }
</style>
