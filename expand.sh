#!/bin/sh

if [ $# -ne 1 ]; then
	echo "usage: $0 <rust source file>"
	exit 1
fi

CMD="rustc +nightly -Zunpretty=expanded -Zparse-only $1 --extern spar_rust=target/debug/libspar_rust.so"
cargo build --quiet \
	&& echo "$CMD" \
	&& $CMD
