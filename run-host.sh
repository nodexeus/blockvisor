#!/bin/bash
set -e
cd luuk/repo
export PATH=$PATH:/root/.cargo/bin
cargo b --release -p blockvisord
target/release/bv node create luukschain
target/release/bv node list -c luukschain
#         list the nodes                            | remove color characters    | skip header|         | extract right info from the line
NODE_ID=$(target/release/bv node list -c luukschain | sed 's/\x1b\[[^\x1b]*m//g' | tail -n +3 | head -1 | awk '{print $1}')
NODE_IP=$(target/release/bv node list -c luukschain | sed 's/\x1b\[[^\x1b]*m//g' | tail -n +3 | head -1 | awk '{print $5}')
echo "Node ID is $NODE_ID, Node IP is $NODE_IP"
target/release/bv node start ${NODE_ID}

scp -r babel -r bv Cargo.lock Cargo.toml root@${NODE_IP}:/root
ssh root@${NODE_IP} -m run-node.sh -o "StrictHostKeyChecking=no"
