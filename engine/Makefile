all:
	cargo build

test:
	cargo test

wasm32:
	cargo build --target wasm32-unknown-unknown --features wasm

testwasm32:
	wasm-pack test --chrome --headless --features wasm
