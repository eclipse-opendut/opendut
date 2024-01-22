#!/bin/sh

set -e
set -x

PEER_ID=f7680e16-fc26-4878-b5c5-fa64fde59b18

cargo ci cleo run -- create peer --name norbert --id $PEER_ID

cargo ci cleo run -- create device --peer-id=$PEER_ID --name="device0" --interface="eth0"
cargo ci cleo run -- create device --peer-id=$PEER_ID --name="device1" --interface="eth1"

cargo ci edgar run -- service --id=$PEER_ID &

sleep 1

CLUSTER_ID=4a872fc8-fd0c-43ec-a9f5-7f7d2e664559

cargo ci cleo run -- create cluster-configuration --cluster-id=$CLUSTER_ID --name="cluster0" --leader-id=$PEER_ID --device-names device0 device1

cargo ci cleo run -- create cluster-deployment --id=$CLUSTER_ID
