#!/bin/bash
set -e
set -x

PKI_ROOT_DIR="/provision/"

"$PKI_ROOT_DIR"/generate-ca.sh
"$PKI_ROOT_DIR"./generate-certificate.sh opendut.local
"$PKI_ROOT_DIR"./generate-certificate.sh auth.opendut.local
"$PKI_ROOT_DIR"./generate-certificate.sh netbird.opendut.local
"$PKI_ROOT_DIR"./generate-certificate.sh netbird-api.opendut.local
"$PKI_ROOT_DIR"./generate-certificate.sh signal.opendut.local
"$PKI_ROOT_DIR"./generate-certificate.sh carl.opendut.local
"$PKI_ROOT_DIR"./generate-certificate.sh nginx-webdav.opendut.local
