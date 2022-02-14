.PHONY: run debug

export RUST_LOG := shva=info

run:
	cargo run

debug:
	RUST_LOG=debug cargo run
