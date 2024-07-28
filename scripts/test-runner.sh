#!/bin/bash

if [ ! -e "regionx-node" ]; then 
    echo "regionx-node binary not found"
    echo "run: cargo build --release --features fast-runtime && cp target/release/regionx-node ."
    exit 1
fi

if [ ! -e "polkadot" ] || [ ! -e "polkadot-parachain" ]; then
    zombienet-linux setup polkadot polkadot-parachain
fi

export PATH=$PWD:$PATH

npm run build

zombienet-linux -p native test $1
