import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri 侧固定 5173，dev 时由 tauri.conf.json 的 devUrl 指向。
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: 'chrome110',
    outDir: 'dist',
    emptyOutDir: true,
  },
});
