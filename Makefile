
.PHONY: run fmt test
run:
	cd renderer; cargo run

fmt:
	cargo fmt --all
	cd renderer; cargo fmt

test:
	cargo test --all
