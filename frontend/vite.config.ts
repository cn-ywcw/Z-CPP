import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 5174 } : undefined,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      },
    },
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
