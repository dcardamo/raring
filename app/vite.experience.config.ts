import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig, type Plugin } from 'vite'
import { fileURLToPath } from 'node:url'
import { existsSync, renameSync } from 'node:fs'

// The unshipped experience prototype: a separate entry that never touches the
// production build. Shipped components keep importing `lib/api`; this build
// gives them the prototype's synthetic data instead of Tauri or the sample fixtures.
const api = fileURLToPath(new URL('./src/lib/api.ts', import.meta.url))
const prototypeApi = fileURLToPath(new URL('./src/experience/api.ts', import.meta.url))

function prototypeData(): Plugin {
  return {
    name: 'raring-prototype-data',
    enforce: 'pre',
    async resolveId(source, importer, options) {
      if (!importer || importer === prototypeApi || !source.endsWith('api')) return null
      const resolved = await this.resolve(source, importer, { ...options, skipSelf: true })
      return resolved?.id.split('?')[0] === api ? prototypeApi : null
    },
    // Serve the prototype from its own directory root.
    closeBundle() {
      const out = fileURLToPath(new URL('./dist-experience/', import.meta.url))
      if (existsSync(out + 'experience.html')) renameSync(out + 'experience.html', out + 'index.html')
    },
  }
}

export default defineConfig({
  plugins: [prototypeData(), svelte()],
  publicDir: 'experience-public',
  server: { open: '/experience.html' },
  build: {
    outDir: 'dist-experience',
    emptyOutDir: true,
    rollupOptions: { input: fileURLToPath(new URL('./experience.html', import.meta.url)) },
  },
})
