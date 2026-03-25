import { onUnmounted } from 'vue';
import { createSSEConnection } from '../api';
import { useLogsStore } from '../stores/logs';
import { useStatsStore } from '../stores/stats';
import type { LogEntry, RequestSummary } from '../types';

export function useSSE(onConnected?: (connected: boolean) => void) {
  const logsStore = useLogsStore();
  const statsStore = useStatsStore();
  let es: EventSource | null = null;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  function connect() {
    if (es) { try { es.close(); } catch {} }
    es = createSSEConnection((event, data) => {
      if (event === 'log') {
        logsStore.addLog(data as LogEntry);
      } else if (event === 'summary') {
        logsStore.upsertRequest(data as RequestSummary);
      } else if (event === 'stats') {
        statsStore.load();
      } else if (event === 'message') {
        // Rust SSE 广播的 refresh 信号
        const d = data as { type?: string };
        if (d?.type === 'refresh') {
          logsStore.loadRequests();
          statsStore.load();
        }
      }
    });

    es.onopen = () => {
      onConnected?.(true);
      // 每 10 秒轮询一次，作为文件监听的兜底
      if (pollTimer) clearInterval(pollTimer);
      pollTimer = setInterval(() => {
        logsStore.loadRequests();
        statsStore.load();
      }, 10000);
    };

    es.onerror = () => {
      onConnected?.(false);
      es?.close();
      es = null;
      reconnectTimer = setTimeout(connect, 3000);
    };
  }

  function disconnect() {
    if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null; }
    if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
    es?.close();
    es = null;
    onConnected?.(false);
  }

  onUnmounted(() => { disconnect(); });

  return { connect, disconnect };
}
