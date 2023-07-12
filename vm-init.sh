cd ~ && \
  sudo apt update && \
  sudo apt --fix-broken install && \ 
  sudo apt install -y bc bison curl clang fish flex git make libelf-dev ccache && \
  curl https://sh.rustup.rs -sSf | bash -s -- -y && \
  echo "EXPORT CCACHE_DIR=/vagrant/.ccache" >> .bashrc && \
  echo "cd /vagrant" >> .bashrc && \
  source "$HOME/.cargo/env" && \
  sudo apt install libssl-dev -y && \
  git clone https://github.com/rust-lang/rust-bindgen -b v0.56.0 --depth 1 && \
  cargo install --path rust-bindgen && \
  cd /vagrant/linux-rust && \
  rustup override set $(scripts/min-tool-version.sh rustc) &&\
  rustup component add rust-src &&\
  make rustavailable
