counter:
	cargo run --package counter --bin counter --features gpu
counter-release:
	cargo run --package counter --bin counter --features gpu --release