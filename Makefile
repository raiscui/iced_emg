# RUSTFLAGS="-Z sanitizer=leak"

editor:
	cargo run --package emg_editor --bin emg_editor --features gpu
add_nodes-release:
	cargo run --package add_nodes --bin add_nodes --features gpu --release
add_nodes:
	RUST_BACKTRACE=full cargo run --package add_nodes --bin add_nodes --features gpu
drag_node-release:
	RUST_BACKTRACE=full cargo run --package drag_node --bin drag_node --features gpu --release
drag_node:
	RUST_BACKTRACE=full cargo run --package drag_node --bin drag_node --features gpu
moving_edge:
	INSTA_FORCE_PASS=1 cargo run --package moving_edge --bin moving_edge --features gpu
moving_edge-dhat:
	cargo run --package moving_edge --bin moving_edge --features gpu --features dhat-heap
moving_edge-release:
	INSTA_FORCE_PASS=1 cargo run --package moving_edge --bin moving_edge --features gpu --release
counter:
	cargo run --package counter --bin counter --features gpu
counter-time:
	cargo build --package counter --bin counter --features gpu --timings
counter-debug:
	cargo run --package counter --bin counter --features gpu --features debug
counter-release:
	cargo run --package counter --bin counter --features gpu --release
line-count:
	git ls-files | xargs cat | wc -l
line-h-count:
	git log --since=2020-01-01 --until=2040-12-31 --pretty=tformat: --numstat | awk '{ add += $1; subs += $2; loc += $1 - $2 } END { printf "added lines: %s, removed lines: %s, current lines: %s\n", add, subs, loc }'
cloc:
	cloc ./
test-native:
	cargo test --workspace --exclude emg_web
test-xx:
	cargo test --package my_package -- --skip my_test
miri:
	INSTA_UPDATE="no" INSTA_OUTPUT="none" MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test