
.PHONY: build-release

start:
	- mkdir -p /tmp/noir-dl
	RUST_LOG=info cargo run -- -n test server --root ../client/build --download-to /tmp/noir-dl/

build-release:
	cargo build --release
