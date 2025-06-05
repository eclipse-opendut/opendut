#!/bin/bash

set -ex  # exit on error, print all commands

opendut-cleo delete cluster-deployment "206e5d0d-029d-4b03-8789-e0ec46e5a6ba"
opendut-cleo delete cluster-descriptor "206e5d0d-029d-4b03-8789-e0ec46e5a6ba"

echo "Listing all peers"
opendut-cleo list peers --output json | jq -r '.[].id'

echo "Deleting all peers"
opendut-cleo delete peer "bcf75b6c-d6e1-42bd-b74e-30690bca88ab"
opendut-cleo delete peer "d629fede-27c8-4270-8e73-f91ae7d31a33"
opendut-cleo delete peer "525b369f-8abb-4b49-8046-25948936ad6c"
opendut-cleo delete peer "8b5835af-0e3c-4a28-a7d7-623a929a0f1b"
opendut-cleo delete peer "a1db14f5-1d08-4876-adf2-ba32d99f25ff"

exit 0
