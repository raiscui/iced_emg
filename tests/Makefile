test-lib:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::test2
graph-build:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::graph_build
bench:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::benchmark
bench-edge:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::benchmark_edge_get
test-all:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release