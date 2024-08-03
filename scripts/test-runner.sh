#!/bin/bash

if [ ! -e "regionx-node" ]; then 
    echo "regionx-node binary not found"
    echo "run: cargo build --release --features fast-runtime && cp target/release/regionx-node ."
    exit 1
fi

zombienet() {
    local ZOMBIENET_COMMAND=$1

    if which zombienet-macos &> /dev/null; then
        zombienet-macos $ZOMBIENET_COMMAND
    elif which zombienet-linux &> /dev/null; then
        zombienet-linux $ZOMBIENET_COMMAND
    elif which zombienet &> /dev/null; then
        zombienet $ZOMBIENET_COMMAND
    else
        echo "Zombienet couldn't be located"
    fi
}

if [ ! -e "polkadot" ] || [ ! -e "polkadot-parachain" ]; then
    zombienet "setup polkadot polkadot-parachain"
fi

export PATH=$PWD:$PATH

npm run build

zombienet "-p native test $1"
