import { defineConfig } from 'vite';
import basicSsl from '@vitejs/plugin-basic-ssl';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [basicSsl()],
  base: '/greenbuttonengine/',
  build: {
    outDir: '../docs',
    emptyOutDir: true,
  },
  server: {
    fs: {
      // Allow serving files from one level up to the project root
      allow: ['.', '../lib/wasm'],
    },
  },
});
