
.PHONY: build-release

start:
	- mkdir -p /tmp/noir-dl
	cargo run -- -n test server --root ../web/build --download-to /tmp/noir-dl/

build-release:
	cargo build --release
