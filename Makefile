moving_edge:
	INSTA_FORCE_PASS=1 cargo run --package moving_edge --bin moving_edge --features gpu
counter:
	cargo run --package counter --bin counter --features gpu
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