cd ~ && \
  sudo apt update && \
  sudo apt --fix-broken install && \ 
  sudo apt install -y bc bison curl clang fish flex git make libelf-dev ccache && \
  curl https://sh.rustup.rs -sSf | bash -s -- -y && \
  echo "export CCACHE_DIR=/vagrant/.ccache" >> .bashrc && \
  echo "cd /vagrant" >> .bashrc && \
  source .bashrc && \
  source "$HOME/.cargo/env" && \
  sudo apt install libssl-dev -y && \
  cargo install --git https://github.com/rust-lang/rust-bindgen --tag v0.65.1 bindgen-cli && \
  cd /vagrant/linux-rust && \
  rustup override set $(scripts/min-tool-version.sh rustc) &&\
  rustup component add rust-src &&\
  make rustavailable
