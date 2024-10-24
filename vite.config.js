import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import wasm from 'vite-plugin-wasm';
import WebWorkerPlugin from 'vite-plugin-webworker-service';

// https://vitejs.dev/config/
export default defineConfig({
  worker: {
    format: 'es',
  },
  plugins: [
    wasm(),
    WebWorkerPlugin(),
    react(),
    {
      name: 'handle-core-js-import',
      enforce: 'pre',
      resolveId(source) {
        if (source === 'core-js/proposals/json-parse-with-source.js') {
          return 'core-js/actual/json/parse';
        }
      }
    }
  ],
  resolve: {
    alias: {
      'core-js/proposals/json-parse-with-source.js': 'core-js/actual/json/parse'
    }
  },
  build: {
    target: 'esnext',
    sourcemap: true,
  },
  optimizeDeps: {
    exclude: ['@provablehq/wasm', '@provablehq/sdk'],
  },
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
