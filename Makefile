
.PHONY: build-release

start:
	cargo run -- -n chrysoberyl server --root ../web/build

build-release:
	cargo build --release
