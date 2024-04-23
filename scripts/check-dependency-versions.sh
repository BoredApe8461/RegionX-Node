#!/bin/bash

output=$(psvm -v "1.6.0")

success="Dependencies in Cargo.toml are already up to date"

if [ "$output" != "$success" ]; then
    exit 1
fi
