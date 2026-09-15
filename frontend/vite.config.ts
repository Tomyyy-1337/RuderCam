import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import compression from 'vite-plugin-compression'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  plugins: [
    svelte(),
    compression({
      algorithm: 'gzip',
      ext: '.gz',
      threshold: 1024,
    }),
  ],
  build: {
    outDir: path.resolve(__dirname, '../backend/static'),
    emptyOutDir: true,
    chunkSizeWarningLimit: 1200,
  },
})