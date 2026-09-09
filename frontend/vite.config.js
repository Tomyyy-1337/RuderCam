import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: path.resolve(__dirname, '../backend/static'),
    emptyOutDir: true,
    chunkSizeWarningLimit: 1200,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (
            id.includes('/src/lib/map.svelte') ||
            id.includes('node_modules/maplibre-gl') ||
            id.includes('node_modules/pmtiles')
          ) {
            return 'map';
          }
        },
      },
    },
  },
})
