#!/bin/bash
set -e

scp -r babel -r bv Cargo.lock Cargo.toml run-node.sh root@host001.blockvisor.dev:/root/luuk/repo
ssh root@host001.blockvisor.dev 'bash -s' < ./run-host.sh
