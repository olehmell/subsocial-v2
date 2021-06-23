#!/usr/bin/env bash
# This script meant to be run on Unix/Linux based systems
set -e

echo "*** Initializing WASM build environment"

CDIR=`dirname "$0"`
export RUSTC_VERSION=`cat $CDIR/../RUSTC_VERSION`

if [ -z $CI_PROJECT_NAME ] ; then
   rustup update $RUSTC_VERSION
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain $RUSTC_VERSION
rustup override set $RUSTC_VERSION
