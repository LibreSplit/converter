# converter

A library for converting [LiveSplit](https://livesplit.org) `.lss` files into the [LibreSplit](https://libresplit.org) `.json` format.

An online converter using this library is available at [libresplit.org/converter](https://libresplit.org/converter).

## Install
```bash
npm i @libresplit/converter
```

## Build
### Prerequisites
- Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- WASM compiler target: `rustup target add wasm32-unknown-unknown`
- wasm-pack: <br>
`cargo install wasm-pack` <br>
 or <br>
`curl https://drager.github.io/wasm-pack/installer/init.sh -sSf | sh`

### Building
```sh
git clone https://github.com/LibreSplit/converter
cd converter
wasm-pack build --scope libresplit --target web --release
```

This outputs a compiled WASM package under `pkg/`.

### Building (for C ABI)
```sh
git clone https://github.com/LibreSplit/converter
cd converter
cargo build --release
```

---

Published as an npm package under [`@libresplit/converter`](https://npmjs.com/package/@libresplit/converter).
