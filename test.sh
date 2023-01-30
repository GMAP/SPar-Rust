#!/bin/bash

cargo test || exit 1
cargo build || exit 1

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}")"  &> /dev/null && pwd)
for directory in "$SCRIPT_DIR"/tests/*; do
	if [ ! -d "$directory" ]; then
		continue
	fi

	PACKAGE="$(basename "$directory")"
	(cargo build --quiet --package "$PACKAGE" && ./target/debug/"$PACKAGE" "$SCRIPT_DIR" && echo "$PACKAGE passed") \
		|| echo "$PACKAGE TEST FAILED"
done
