import * as progenitor from 'progenitor'

// Required to see rust panic message and backtrace on JS console.
// (Without it we only get the JS backtrace, saying "unreachable executed".)
progenitor.set_panic_hook()

if (progenitor.is_debug_build()) {
    console.warn(
        'the rust wasm module was built in debug mode and will run ~100x slower',
    )
}
