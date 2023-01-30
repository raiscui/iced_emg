test-wasm:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release
bench:
	cargo criterion
pc-bench:
	cargo test --package emg_animation --test pcbench --release  -- pc_bench --nocapture