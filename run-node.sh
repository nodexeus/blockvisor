#!/bin/bash
set -e


curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
source "$HOME/.cargo/env"
apt install gcc -y
cargo b --release -p babel --features vsock
systemctl stop babel
cp target/release/babel /usr/bin/babel
systemctl start babel
journalctl -b -u babel
