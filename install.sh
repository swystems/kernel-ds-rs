#!/bin/bash

sudo apt update && \
	sudo NEEDRESTART_MODE=a apt install -y build-essential bc bison curl fish flex git make libelf-dev libssl-dev clang-15 llvm-15-dev lld-15 && \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.70.0 --component rust-src && \
	source ~/.cargo/env && \
	cargo install bindgen --vers 0.56.0 && \
	cd /linux_src && make LLVM=-15 rustavailable && \
	sudo make headers_install && sudo make modules_install && sudo make install


