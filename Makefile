format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet

test:
	cargo test --quiet
all: 
	format lint test