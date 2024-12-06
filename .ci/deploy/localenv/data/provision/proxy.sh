#!/bin/bash

# Proxy configuration script for testing the localenv in a virtual machine

PROXY="$1"
if [ -z "$PROXY" ]; then
  echo "# Please provide your proxy configuration as argument in the following format:"
  echo "source ${BASH_SOURCE[0]} http://192.168.56.1:3128"
  echo "# Beware: The proxy address must be valid on the host and within the virtual machine!"
else
  echo "setting proxy server $PROXY"

  export http_proxy=$PROXY
  export https_proxy=$PROXY
  export VAGRANT_HTTP_PROXY=$PROXY
  export VAGRANT_HTTPS_PROXY=$PROXY

  TESTENV_HOSTS="opendut.local,auth.opendut.local,loki,keycloak,prometheus,tempo,promtail,nginx-webdav,otel-collector,netbird-management,localhost:8080"
  # 192.168.56.0/24,192.168.32.0/24,192.168.33.0/24
  TESTENV_NETWORKS="192.168.0.0/16"

  export VAGRANT_NO_PROXY=localhost,127.0.0.1,$TESTENV_HOSTS,$TESTENV_NETWORKS
  export no_proxy=localhost,127.0.0.1,$TESTENV_HOSTS,$TESTENV_NETWORKS
  export NO_PROXY=localhost,127.0.0.1,$TESTENV_HOSTS,$TESTENV_NETWORKS
fi

