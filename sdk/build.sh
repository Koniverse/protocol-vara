#!/bin/bash
set -e 

yarn wasm:build
./package-wasm.sh

yarn install
yarn lint
yarn contract:build
yarn erc-20:generate
yarn invariant:generate
yarn fix-generate
yarn build
yarn postbuild