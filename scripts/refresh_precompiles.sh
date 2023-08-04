#!/bin/bash
set -xe

SRC_DIR=etc/system-contracts/artifacts-zk/cache-zk/solpp-generated-contracts
DST_DIR=src/deps/contracts/

mkdir -p $DST_DIR


precompiles=("Ecrecover" "Keccak256" "SHA256" "ModExp" "EcAdd" "EcMul" "EcPairing" "Playground")

for precompile in "${precompiles[@]}"; do
    cp etc/system-contracts/contracts/precompiles/artifacts/$precompile.yul/$precompile.yul.zbin $DST_DIR
done
