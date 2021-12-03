#!/bin/bash

if [ ! -d ".out" ]; then
  mkdir .out
fi
if [ ! -f "dm.out" ]; then
  cd .out && touch dm.out && cd ..
fi
cddm|| exit
echo "Compiling dm..."
date >> ../.out/ dm.out
cargo fmt
cargo clippy
cargo build &> ../.out/dm.out