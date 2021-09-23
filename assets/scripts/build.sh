#!/bin/bash

if [ ! -d ".out" ]; then
  mkdir .out
fi
if [ ! -f "dmd.out" ]; then
  cd .out && touch dmd.out && cd ..
fi
cd dmd || exit
echo "Compiling dmd..."
date >> ../.out/dmd.out
cargo fmt
cargo clippy
cargo build &> ../.out/dmd.out