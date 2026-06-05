import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  plugins: [react()],
  server: { port: 5173 },
  resolve: {
    alias: {
      '@leds/core': fileURLToPath(new URL('../packages/core/src/browser.ts', import.meta.url)),
    },
  },
});
