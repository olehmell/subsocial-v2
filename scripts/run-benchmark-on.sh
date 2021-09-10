#!/usr/bin/env bash

set -e

SCRIPT_DIR=$(dirname "$0")
ROOT_DIR=$SCRIPT_DIR/..

if [[ -z $1 ]]; then
  echo "You have to specify pallet name"
  echo "For example: ./run-benchmark-on.sh pallet_spaces"
  exit 1
fi

PALLET_NAME="$1"

"$ROOT_DIR"/target/release/subsocial-node benchmark \
  --chain dev \
  --execution wasm \
  --wasm-execution Compiled \
  --pallet "$PALLET_NAME" \
  --extrinsic '*' \
  --steps 50 \
  --repeat 20 \
  --heap-pages 4096 \
  --output ./pallets/claims/src/weights.rs \
  --template ./.maintain/weight-template.hbs
