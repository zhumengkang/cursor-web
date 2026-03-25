import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// RUST_UI=1 时：build 到 Rust static 目录，base 为 /
// 默认（Node.js 模式）：build 到 ../public/vue，base 为 /public/vue/
const rustMode = process.env.RUST_UI === '1';
const isProd = process.env.NODE_ENV === 'production';

export default defineConfig({
  plugins: [vue()],
  base: rustMode ? '/' : (isProd ? '/public/vue/' : '/'),
  build: {
    outDir: rustMode ? '../../cursor2api-rust/static' : '../public/vue',
    emptyOutDir: true,
  },
  server: {
    port: 5173,
    proxy: {
      // 开发模式：代理到 Rust UI 后端（3001）
      '/api': { target: 'http://localhost:3001', changeOrigin: true },
    },
  },
});
