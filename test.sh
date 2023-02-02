#!/bin/bash

cargo test || exit 1
cargo build || exit 1

ERRORS=0
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}")"  &> /dev/null && pwd)
for directory in "$SCRIPT_DIR"/tests/*; do
	if [ ! -d "$directory" ]; then
		continue
	fi

	PACKAGE="$(basename "$directory")"
	if ! cargo build --quiet --package "$PACKAGE"; then
		echo "$PACKAGE TEST COMPILATION FAILED"
		ERRORS=$((ERRORS + 1))
		continue
	fi

	if ! ./target/debug/"$PACKAGE" "$SCRIPT_DIR"; then
		echo "$PACKAGE TEST EXECUTION FAILED"
		ERRORS=$((ERRORS + 1))
		continue
	fi

	echo "$PACKAGE test passed"
done

exit $ERRORS
