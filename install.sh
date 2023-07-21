#!/bin/bash

sudo apt update && sudo apt install -y bc bison curl fish flex git make libelf-dev rename libssl-dev

# LLVM
cd ~/
wget https://apt.llvm.org//llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 15
rm llvm.sh
sudo mkdir /usr/bin/llvm
sudo cp /usr/bin/*-15 /usr/bin/llvm/
cd /usr/bin/llvm
sudo rename 's/-15//' ./*
echo 'export PATH=$PATH:/usr/bin/llvm'

# rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
mkdir ~/git-repo
source ~/.cargo/env
rustup override set 1.7.0
rustup component add rust-src
cd ~/git-repo
git clone https://github.com/rust-lang/rust-bindgen -b v0.56.0 --depth=1
cargo install --path rust-bindgen

cd /linux_src && make LLVM=1 rustavailable


