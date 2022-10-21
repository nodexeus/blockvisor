#!/bin/bash
set -e


curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal && \
    source "$HOME/.cargo/env" && \
    curl https://snapshots.helium.wtf/genesis.mainnet > /h && \
    miner genesis load /h && \
    apt install gcc -y && \
    cargo b -p babel --features vsock && \
    systemctl stop babel && \
    cp target/debug/babel /usr/bin/babel && \
    systemctl start babel && \
    journalctl -rb -u babel
