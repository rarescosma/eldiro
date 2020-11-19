#!/bin/bash
set -o errexit

cargo build --release
sudo setcap cap_net_admin=eip $CARGO_RELEASE_DIR/trust
"$CARGO_RELEASE_DIR/trust" &
pid=$!
sudo ip addr add 10.5.5.5/24 dev tun0
sudo ip link set up dev tun0
trap 'kill $pid' INT TERM EXIT
wait $pid
