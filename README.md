# Progenitor

This project is:

* a testbed for growth of hexagonal cells
* a cellular automaton with rules to be evolved
* an open-ended playground for some ideas
* a mess while I'm figuring out Rust

This repo is Rust library with bindings for Python ([PyO3](https://pyo3.rs)) and Typescript ([wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/)).

Build snapshots/demos: [log2.ch/progenitor](https://log2.ch/progenitor/)

Named after [progenitor cells](https://en.wikipedia.org/wiki/Progenitor_cell).

## Build and Run

[![Build Status](https://travis-ci.org/martinxyz/progenitor.svg?branch=master)](https://travis-ci.org/martinxyz/progenitor)

### Rust Library

Run `cargo build` and `cargo test`.

TODO: how to create an optimized build

### Webassembly Module

Install [wasm-pack](https://rustwasm.github.io/wasm-pack/), e.g. `cargo install wasm-pack`.

```bash
wasm-pack build
```

This will create a node package in the *pkg* directory.

### Web Application

After the steps above, build and run the web application:

```bash
cd www
npm install
npm start
```

Then open [http://localhost:8080](http://localhost:8080) in a browser.

### Python Module

Install [maturin](https://github.com/PyO3/maturin), e.g. `pip3 install
maturin`. Inside a virtualenv, run:

```bash
maturin develop --cargo-extra-args="--features=python"
```

Or, for an optimized build:

```bash
RUSTFLAGS="-C target-cpu=native" \
    maturin develop --release --cargo-extra-args="--features=python"
```

This will install the Python module into the virtualenv. You can then use
`import progenitor` from Python.

Note: if you are running Python from the toplevel directory this will actually
import the *progenitor* subdirectory, instead of the package installed into the
virtualenv. This works because `maturin develop` also copies the compiled Rust
extension into that directory.
