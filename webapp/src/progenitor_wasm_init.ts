import * as progenitor from 'progenitor'

// from https://github.com/vitejs/vite/issues/4551#issuecomment-983012078
import init from 'progenitor';
import wasmUrl from 'progenitor/progenitor_wasm_bg.wasm?url'
await init(wasmUrl)

// Required to see rust panic message and backtrace on JS console.
// (Without it we only get the JS backtrace, saying "unreachable executed".)
progenitor.set_panic_hook()

if (progenitor.is_debug_build()) {
    console.warn(
        'the rust wasm module was built in debug mode and will run ~100x slower',
    )
}
