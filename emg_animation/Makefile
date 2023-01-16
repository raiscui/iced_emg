test-wasm:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release
bench:
	cargo criterion
pcbench:
	cargo test --package emg_animation --test pcbench --release  -- pc_bench --nocapture