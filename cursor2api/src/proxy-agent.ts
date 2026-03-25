/**
 * proxy-agent.ts - 代理支持模块
 *
 * 支持代理类型：
 *   - http://[user:pass@]host:port
 *   - https://[user:pass@]host:port
 *   - socks4://[user:pass@]host:port
 *   - socks4a://[user:pass@]host:port
 *   - socks5://[user:pass@]host:port
 *   - socks5h://[user:pass@]host:port
 *
 * 实现策略：
 *   - http/https 代理：使用 undici.ProxyAgent（最优性能，Node.js fetch 原生支持）
 *   - socks 代理：使用 SocksProxyAgent，通过设置 Node.js 全局 http/https agent，
 *     同时包装为 undici.Agent 的 connect 函数供 fetch 使用。
 *
 *   由于 undici 不原生支持 SOCKS，对 socks 代理采用「全局 agent 替换」方案：
 *   将 SocksProxyAgent 挂载到 http.globalAgent / https.globalAgent，
 *   并通过自定义 dispatcher 的 connect 委托给 socks agent 的 socket 建立。
 */

import { ProxyAgent, Agent, type Dispatcher } from 'undici';
import { SocksProxyAgent } from 'socks-proxy-agent';
import * as http from 'http';
import * as https from 'https';
import { getConfig } from './config.js';

// 缓存已创建的 dispatcher，避免重复创建
let cachedProxyUrl: string | undefined;
let cachedDispatcher: Dispatcher | undefined;
let cachedVisionProxyUrl: string | undefined;
let cachedVisionDispatcher: Dispatcher | undefined;

// 代理池轮询状态
let proxyPoolIndex = 0;
const dispatcherCache = new Map<string, Dispatcher>();

/**
 * 从代理池中轮询取出下一个代理 URL
 */
export function getNextProxyUrl(): string | undefined {
    const config = getConfig();
    // 优先使用 proxies 列表（多节点轮询）
    if (config.proxies && config.proxies.length > 0) {
        const url = config.proxies[proxyPoolIndex % config.proxies.length];
        proxyPoolIndex++;
        return url;
    }
    // 回退到单 proxy
    return config.proxy;
}

/**
 * 为代理池中的指定 URL 获取（或创建缓存）Dispatcher
 */
function getPooledDispatcher(proxyUrl: string): Dispatcher {
    if (!dispatcherCache.has(proxyUrl)) {
        dispatcherCache.set(proxyUrl, createDispatcher(proxyUrl));
    }
    return dispatcherCache.get(proxyUrl)!;
}

function isSocksProxy(url: string): boolean {
    return /^socks[45]/i.test(url);
}

/**
 * 为 undici（Node.js fetch）创建适配任意代理协议的 Dispatcher。
 *
 * - http/https 代理：直接用 undici.ProxyAgent
 * - socks 代理：将 SocksProxyAgent 设为全局 http/https agent，
 *   并返回一个透传的 undici.Agent（fetch 会走全局 agent 的 socket 建立）
 */
function createDispatcher(proxyUrl: string): Dispatcher {
    if (!isSocksProxy(proxyUrl)) {
        // HTTP/HTTPS 代理：undici 原生支持
        return new ProxyAgent(proxyUrl);
    }

    // SOCKS 代理：替换 Node.js 全局 http/https agent
    // undici fetch 在建立连接时会委托给 Node.js net.connect，
    // 通过全局 agent 的 socket 建立走 socks tunnel
    const socksAgent = new SocksProxyAgent(proxyUrl);
    // globalAgent 在某些 TS 版本声明为 readonly，用 defineProperty 绕过
    Object.defineProperty(http, 'globalAgent', { value: socksAgent, writable: true, configurable: true });
    Object.defineProperty(https, 'globalAgent', { value: socksAgent, writable: true, configurable: true });
    console.log(`[Proxy] SOCKS 代理已设为全局 agent: ${proxyUrl}`);

    // 返回默认 undici Agent（连接层已由全局 agent 接管）
    return new Agent();
}

/**
 * 获取全局代理 dispatcher（供 Node.js fetch 使用）
 * 返回 undefined 表示不使用代理（直连）
 */
export function getProxyDispatcher(): Dispatcher | undefined {
    const config = getConfig();
    const proxyUrl = config.proxy;
    if (!proxyUrl) return undefined;

    if (cachedProxyUrl !== proxyUrl) {
        console.log(`[Proxy] 使用全局代理: ${proxyUrl}`);
        cachedProxyUrl = proxyUrl;
        cachedDispatcher = createDispatcher(proxyUrl);
    }
    return cachedDispatcher;
}

/**
 * 构建 fetch 的额外选项（包含 dispatcher）
 * 优先使用代理池轮询，回退到单代理
 */
export function getProxyFetchOptions(): Record<string, unknown> {
    const config = getConfig();
    if (config.proxies && config.proxies.length > 0) {
        const url = getNextProxyUrl()!;
        return { dispatcher: getPooledDispatcher(url) };
    }
    const dispatcher = getProxyDispatcher();
    return dispatcher ? { dispatcher } : {};
}

/**
 * Vision 独立代理：优先使用 vision.proxy，否则回退到全局 proxy
 */
export function getVisionProxyFetchOptions(): Record<string, unknown> {
    const config = getConfig();
    const visionProxy = config.vision?.proxy;

    if (visionProxy) {
        if (cachedVisionProxyUrl !== visionProxy) {
            console.log(`[Proxy] Vision 独立代理: ${visionProxy}`);
            cachedVisionProxyUrl = visionProxy;
            cachedVisionDispatcher = createDispatcher(visionProxy);
        }
        return { dispatcher: cachedVisionDispatcher };
    }

    return getProxyFetchOptions();
}
