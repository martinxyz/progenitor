# Progenitor

A testbed for rule-based growth of hexagonal cells.

A 2D cellular automaton with programmable rules, to be evolved. 

An open-ended experiment and playground for some ideas.

This is a Rust library with bindings for Python (native module with
[PyO3](https://pyo3.rs)) and Javascript (Webassembly with
[wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/)).

Named after [progenitor cells](https://en.wikipedia.org/wiki/Progenitor_cell).

# Build and Run

## Rust Library

Run `cargo build` and `cargo test`.

TODO: how to create an optimized build

## Python Bindings

Install [maturin](https://github.com/PyO3/maturin), e.g. `pip3 install
maturin`. Inside a virtualenv, run:

```bash
maturin develop
```

Or, for an optimized build:

```bash
RUSTFLAGS="-C target-cpu=native" maturin develop --release
```

This will install the Python module into the virtualenv. You can then use
`import progenitor` from Python.

Note: if you are running Python from the toplevel directory this will actually
import the *progenitor* subdirectory, instead of the package installed into the
virtualenv. This works because `maturin develop` also copies the compiled Rust
extension into that directory.
