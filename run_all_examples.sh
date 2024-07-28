#!/bin/bash
output=$(cargo build --example 2>&1)
 # sed to trim whitespace

for example in $(echo "$output" | tail -n +3 | sed 's/^[ \t]*//;s/[ \t]*$//')
do
    cargo run --example "$(basename "$example")" -- $args
done