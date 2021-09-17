
.PHONY: build-release

start:
	cargo run -- -n test server --root ../web/build

build-release:
	cargo build --release
