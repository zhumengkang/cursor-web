import type { LogEntry, RequestSummary, Stats, Payload, HotConfig, SaveConfigResult } from './types';
import { useAuthStore } from './stores/auth';
import { getActivePinia } from 'pinia';

function getAuthHeader(): Record<string, string> {
  const token = localStorage.getItem('cursor2api_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function apiFetch<T>(path: string): Promise<T> {
  const res = await fetch(path, { headers: getAuthHeader() });
  if (res.status === 401) {
    const pinia = getActivePinia();
    if (pinia) useAuthStore(pinia).logout();
    throw new Error('HTTP 401');
  }
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<T>;
}

export async function fetchLogs(params?: { requestId?: string; since?: number }): Promise<LogEntry[]> {
  const q = new URLSearchParams();
  if (params?.requestId) q.set('requestId', params.requestId);
  if (params?.since != null) q.set('since', String(params.since));
  const qs = q.toString() ? '?' + q.toString() : '';
  const res = await apiFetch<LogEntry[] | { logs: LogEntry[] }>(`/api/logs${qs}`);
  return Array.isArray(res) ? res : res.logs ?? [];
}

export async function fetchRequests(limit = 50): Promise<RequestSummary[]> {
  const res = await apiFetch<RequestSummary[] | { requests: RequestSummary[] }>(`/api/requests?limit=${limit}`);
  return Array.isArray(res) ? res : res.requests ?? [];
}

export function fetchStats(since?: number): Promise<Stats> {
  const qs = since !== undefined ? `?since=${since}` : '';
  return apiFetch<Stats>(`/api/vue/stats${qs}`);
}

export async function fetchPayload(requestId: string): Promise<Payload> {
  const res = await apiFetch<{ payload: Payload }>(`/api/payload/${requestId}`);
  return res.payload;
}

export async function clearLogs(): Promise<void> {
  const res = await fetch('/api/logs/clear', {
    method: 'POST',
    headers: getAuthHeader(),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
}

export async function fetchConfig(): Promise<HotConfig> {
  const res = await apiFetch<{ config: HotConfig } | HotConfig>('/api/config');
  return (res as { config: HotConfig }).config ?? (res as HotConfig);
}

export interface ModelInfo {
  id: string;
  label: string;
  provider: string;
}

export function fetchModels(): Promise<{ models: ModelInfo[] }> {
  return apiFetch<{ models: ModelInfo[] }>('/api/models');
}

export async function saveConfig(cfg: Partial<HotConfig>): Promise<SaveConfigResult> {
  const res = await fetch('/api/config', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...getAuthHeader() },
    body: JSON.stringify(cfg),
  });
  if (res.status === 401) {
    const pinia = getActivePinia();
    if (pinia) useAuthStore(pinia).logout();
    throw new Error('HTTP 401');
  }
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<SaveConfigResult>;
}

export interface RequestsPage {
  summaries: RequestSummary[];
  hasMore: boolean;
  total: number;
  statusCounts: Record<string, number>;
}

export interface RequestsFilter {
  limit?: number;
  before?: number;
  status?: string;
  keyword?: string;
  since?: number;
}

export async function fetchMoreRequests(filter: RequestsFilter = {}): Promise<RequestsPage> {
  const q = new URLSearchParams({ limit: String(filter.limit ?? 50) });
  if (filter.before !== undefined) q.set('before', String(filter.before));
  if (filter.since !== undefined) q.set('since', String(filter.since));
  if (filter.status) q.set('status', filter.status);
  if (filter.keyword) q.set('keyword', filter.keyword);
  const res = await apiFetch<{ requests?: RequestSummary[]; summaries?: RequestSummary[]; hasMore: boolean; total?: number; statusCounts?: Record<string, number> }>(`/api/requests/more?${q.toString()}`);
  return {
    summaries: res.summaries ?? res.requests ?? [],
    hasMore: res.hasMore,
    total: res.total ?? (res.summaries ?? res.requests ?? []).length,
    statusCounts: res.statusCounts ?? { all: 0, success: 0, degraded: 0, error: 0, processing: 0, intercepted: 0 },
  };
}

// ==================== API Keys ====================
import type { ApiKey, CreateKeyRequest, UpdateKeyRequest, KeyStats } from './types';

export async function fetchKeys(): Promise<ApiKey[]> {
  const res = await apiFetch<{ keys: ApiKey[] }>('/api/keys');
  return res.keys;
}

export async function createKey(req: CreateKeyRequest): Promise<ApiKey> {
  const res = await fetch('/api/keys', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...getAuthHeader() },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error((err as any).error || `HTTP ${res.status}`);
  }
  const data = await res.json() as { key: ApiKey };
  return data.key;
}

export async function updateKey(id: string, req: UpdateKeyRequest): Promise<ApiKey> {
  const res = await fetch(`/api/keys/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json', ...getAuthHeader() },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<ApiKey>;
}

export async function deleteKey(id: string): Promise<void> {
  const res = await fetch(`/api/keys/${id}`, {
    method: 'DELETE',
    headers: getAuthHeader(),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
}

export interface StatsFilter {
  since?: number;
  until?: number;
  granularity?: 'hour' | 'day' | 'week' | 'month';
  key?: string;
}

export async function fetchKeyStats(filter: StatsFilter = {}): Promise<KeyStats> {
  const q = new URLSearchParams();
  if (filter.since !== undefined) q.set('since', String(filter.since));
  if (filter.until !== undefined) q.set('until', String(filter.until));
  if (filter.granularity) q.set('granularity', filter.granularity);
  if (filter.key) q.set('key', filter.key);
  const qs = q.toString() ? '?' + q.toString() : '';
  const res = await apiFetch<{ stats: KeyStats }>(`/api/keys/stats${qs}`);
  return res.stats;
}

// ==================== 代理节点 ====================

export interface ProxiesResponse {
  proxies: string[];
  proxy?: string;
}

export interface TestProxyResponse {
  ok: boolean;
  latency?: number;
  status?: number;
  error?: string;
}

export async function fetchProxies(): Promise<ProxiesResponse> {
  return apiFetch<ProxiesResponse>('/api/proxies');
}

export async function saveProxies(proxies: string[]): Promise<ProxiesResponse> {
  const res = await fetch('/api/proxies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...getAuthHeader() },
    body: JSON.stringify({ proxies }),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<ProxiesResponse>;
}

export async function testProxy(proxy: string, target?: string): Promise<TestProxyResponse> {
  const res = await fetch('/api/proxies/test', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...getAuthHeader() },
    body: JSON.stringify({ proxy, target }),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<TestProxyResponse>;
}

export function createSSEConnection(onMessage: (event: string, data: unknown) => void): EventSource {
  const token = localStorage.getItem('cursor2api_token');
  const url = token ? `/api/logs/stream?token=${encodeURIComponent(token)}` : '/api/logs/stream';
  const es = new EventSource(url);
  es.onmessage = (e) => {
    try { onMessage('message', JSON.parse(e.data)); } catch { /* ignore */ }
  };
  const events = ['log', 'summary', 'stats'];
  for (const ev of events) {
    es.addEventListener(ev, (e) => {
      try { onMessage(ev, JSON.parse((e as MessageEvent).data)); } catch { /* ignore */ }
    });
  }
  return es;
}
