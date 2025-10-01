#!/bin/bash

# e.g. ARGS="-A -T4 127.0.0.1": nmap -A -T4 127.0.0.1 -oN /results/nmap_results.txt
nmap "$@" -oN /results/nmap_results.txt
touch /results/.results_ready
sleep infinity
