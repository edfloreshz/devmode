#!/bin/bash

if [ ! -d ".out" ]; then
  mkdir .out
fi
if [ ! -f "dmd.out" ] || [ ! -f "dmdt.out" ] || [ ! -f "dmdlib.out" ]; then
  cd .out && touch dmd.out dmdt.out dmdlib.out && cd ..
fi
cd dmd || exit
echo "Compiling dmd..."
date >> ../.out/dmd.out
cargo fmt
cargo clippy
cargo build &> ../.out/dmd.out

cd ../dmdt || exit
echo "Compiling dmdt..."
date >> ../.out/dmdt.out
cargo fmt
cargo clippy
cargo build &> ../.out/dmdt.out

cd ../dmdlib || exit
echo "Compiling dmdlib..."
date >> ../.out/dmdlib.out
cargo fmt
cargo clippy
cargo build &> ../.out/dmdlib.out
