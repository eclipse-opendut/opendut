#!/bin/bash

cd /usr/local/bin

# curl cleo
curl https://carl.opendut.local/api/cleo/x86_64-unknown-linux-gnu/download --output cleo.tar.gz
tar --strip-components=1 -xvf cleo.tar.gz

sleep infinity