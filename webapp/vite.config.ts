import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        // hot-reload breaks when the wasm module got recompiled
        svelte({ hot: false }),
        wasm(),
        // Removing this plugin also fixes the interaction problem:
        topLevelAwait(),
    ],
    base: '',
    build: {
        // Add this before removing topLevelAwait() (doesn't fix the problem on its own):
        // target: 'es2022',
        minify: true,
        sourcemap: true,
    },
})
