#!/bin/env bash
cargo b --release
sudo setcap cap_net_admin=eip target/release/tcp_proto
ext=$?
echo "$ext"
if [[ $ext -ne 0 ]]; then
    exit $ext
fi
./target/release/tcp_proto &
pid=$!
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0
trap "kill $pid" INT TERM
wait $pid