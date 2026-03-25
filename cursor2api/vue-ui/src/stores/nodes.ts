import { defineStore } from 'pinia';
import { ref } from 'vue';

export type TestStatus = 'idle' | 'testing' | 'ok' | 'fail';

export interface ProxyTestResult {
  proxy: string;
  status: TestStatus;
  latency: number | null;
  httpStatus: number | null;
  error: string | null;
  testedAt: number | null;
}

const LS_KEY = 'proxy_test_results';

function loadResults(): Map<string, ProxyTestResult> {
  try {
    const raw = localStorage.getItem(LS_KEY);
    if (raw) return new Map(JSON.parse(raw));
  } catch {}
  return new Map();
}

function saveResults(map: Map<string, ProxyTestResult>) {
  try {
    localStorage.setItem(LS_KEY, JSON.stringify(Array.from(map.entries())));
  } catch {}
}

export const useNodesStore = defineStore('nodes', () => {
  const results = ref<Map<string, ProxyTestResult>>(loadResults());

  function getResult(proxy: string): ProxyTestResult {
    return results.value.get(proxy) ?? {
      proxy,
      status: 'idle',
      latency: null,
      httpStatus: null,
      error: null,
      testedAt: null,
    };
  }

  function setResult(r: ProxyTestResult) {
    results.value.set(r.proxy, r);
    saveResults(results.value);
  }

  function clearResult(proxy: string) {
    results.value.delete(proxy);
    saveResults(results.value);
  }

  function clearAll() {
    results.value.clear();
    saveResults(results.value);
  }

  return { results, getResult, setResult, clearResult, clearAll };
});
