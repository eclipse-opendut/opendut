#!/bin/bash

nmap "$@" -oN /results/nmap_results.txt
touch /results/.results_ready
sleep infinity
