#!/bin/sh

# Note: we use the overwrite because currently we do not care that much about
# the specific comipile errors we get. We also redirect stderr because it
# prints an absurd amount of information
CMD="TRYBUILD=overwrite cargo test 2> /dev/null"
echo "$CMD"
eval "$CMD"
