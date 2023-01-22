counter:
	cargo run --package counter --bin counter --features gpu
counter-debug:
	cargo run --package counter --bin counter --features gpu --features debug
counter-release:
	cargo run --package counter --bin counter --features gpu --release
line-count:
	git ls-files | xargs cat | wc -l