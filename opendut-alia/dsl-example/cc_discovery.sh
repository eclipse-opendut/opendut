#!/bin/bash

usage() { echo "Usage: $0 [-i <CAN_INTERFACE>]" 1>&2; exit 1;}

# arg1 = CAN_INTERFACE
CAN_INTERFACE=""

while getopts ":i:" opt; do
  case "$opt" in
    i)
      CAN_INTERFACE=${OPTARG}
      ;;
    :)
      echo "Error: ${OPTARG} argument missing."
      usage
      exit 1
      ;;
    *)
      usage
      exit 1
      ;;
    \?)
      echo "Error: Syntax Invalid"
      usage
      exit 1
      ;;
  esac
done
shift $((OPTIND-1))

#if [ -z "${CAN_INTERRACE}"]; then
# usage
#fi

# Redirect to path
cd /home/kali/Documents/CAN/CC/caringcaribou/

# Get Arbitration IDs from discovery
cc.py -i "$CAN_INTERFACE" uds discovery

echo "Finished execution!"
