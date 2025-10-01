#!/bin/bash

# configure http proxy for Docker in Docker container (cruizba/ubuntu-dind)
# see also https://docs.docker.com/engine/cli/proxy/
if [ -n "$HTTP_PROXY" ]; then
  echo "Setting up Docker proxy configuration with HTTP_PROXY=$HTTP_PROXY"
  mkdir -p "$HOME"/.docker/ && echo "{ \"proxies\": { \"default\": { \"httpProxy\": \"$HTTP_PROXY\", \"httpsProxy\": \"$HTTP_PROXY\", \"noProxy\": \"$NO_PROXY\" } }}" > "$HOME"/.docker/config.json
fi

