#!/bin/bash

cargo b
sudo setcap cap_net_admin=eip ~/Documents/TCP_RUST/trust/target/debug/trust
~/Documents/TCP_RUST/trust/target/debug/trust &
pid=$!
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0
trap "kill $pid" INT TERM
wait $pid
