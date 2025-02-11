#!/bin/bash

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

  TESTENV_NETBIRD_HOSTS="netbird-management,netbird-dashboard,netbird-signal,netbird-coturn"
  TESTENV_EDGAR_HOSTS="edgar-leader,edgar-*"
  TESTENV_HOSTS="$TESTENV_NETBIRD_HOSTS,keycloak,keycloak.internal,$TESTENV_EDGAR_HOSTS,carl,loki,tempo,prometheus,promtail,otel-collector,grafana,nginx-webdav"
  # 192.168.56.0/24,192.168.32.0/24,192.168.33.0/24
  TESTENV_NETWORKS="192.168.56.10,192.168.0.0/16"

  export VAGRANT_NO_PROXY=localhost,127.0.0.1,$TESTENV_HOSTS,$TESTENV_NETWORKS
  export no_proxy=localhost,127.0.0.1,$TESTENV_HOSTS,$TESTENV_NETWORKS
  export NO_PROXY=localhost,127.0.0.1,$TESTENV_HOSTS,$TESTENV_NETWORKS
fi

