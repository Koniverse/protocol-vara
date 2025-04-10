#!/bin/bash
set -e 

yarn install
yarn lint
yarn wasm:build
./package-wasm.sh
yarn contract:build
yarn erc-20:generate
yarn invariant:generate
yarn fix-generate
yarn build