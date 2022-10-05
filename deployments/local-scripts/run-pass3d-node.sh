#!/usr/bin/env bash

# Run a development instance of the Rialto Substrate bridge node.
# To override the default port just export RIALTO_PORT=9944

PASS3D_PORT="${PASS3D_PORT:-9944}"

RUST_LOG=runtime=trace \
    ./target/release/pass3d-bridge-node --dev --tmp \
    --rpc-cors=all --unsafe-rpc-external --unsafe-ws-external \
    --port 33033 --rpc-port 9933 --ws-port $PASS3D_PORT -lruntime::bridge-grandpa=debug \
