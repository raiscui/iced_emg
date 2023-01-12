tongji:
	git log --author="崔路明" --pretty=tformat: --numstat | awk '{ add += $1; subs += $2; loc += $1 - $2 } END { printf "added lines: %s, removed lines: %s, total lines: %s\n", add, subs, loc }' -
counter:
	cargo run --package counter --bin counter --features gpu --features debug