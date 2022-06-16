test-lib:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::test2
graph-build:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::new_graph_build_test
bench-new:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::benchmark_new_graph_build
bench-new-rc:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::benchmark_new_rc_graph_build
test-new-rc:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --lib -- wasm_test::test_new_rc_graph_build
bench:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::benchmark
bench2:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::b2enchmark
bench-edge:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::x_benchmark_edge_get
test-all:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release

new-old:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::new_old_comp
new-str:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::new_str
new-text:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::render_new
old-str:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::old_str
old-text:
	wasm-pack test --chrome --chromedriver /opt/homebrew/bin/chromedriver --release --lib -- wasm_test::render_old
