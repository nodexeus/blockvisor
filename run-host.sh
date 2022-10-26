#!/bin/bash
set -e

ssh root@host001.blockvisor.dev
cd luuk/repo &&\
    export PATH=$PATH:/root/.cargo/bin &&\
    cargo b -p blockvisord &&\
    make reinstall &&\
    bv node create chainia &&\
    bv node list -c chainia &&\
    #         list the nodes                            | remove color characters    | skip header|         | extract right info from the line
    NODE_ID=$(bv node list -c chainia | sed 's/\x1b\[[^\x1b]*m//g' | tail -n +3 | head -1 | awk '{print $1}') &&\
    NODE_IP=$(bv node list -c chainia | sed 's/\x1b\[[^\x1b]*m//g' | tail -n +3 | head -1 | awk '{print $5}') &&\
    echo "Node ID is $NODE_ID, Node IP is $NODE_IP" &&\
    bv node start ${NODE_ID}

scp -r babel bv Cargo.lock Cargo.toml root@${NODE_IP}:/root
ssh root@${NODE_IP} -o "StrictHostKeyChecking=no"
