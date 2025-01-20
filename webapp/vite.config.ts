import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import wasm from 'vite-plugin-wasm'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        // hot-reload breaks when the wasm module got recompiled
        svelte({ compilerOptions: {hmr : false} }),
        wasm(),
    ],
    base: '',
    build: {
        target: 'es2022',
        minify: true,
        sourcemap: true,
    },
    worker: {
        format: 'es',
        plugins: () => [
            wasm(),
        ]
    }
})
