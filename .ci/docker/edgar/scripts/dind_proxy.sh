#!/bin/bash

# configure http proxy for Docker in Docker container (cruizba/ubuntu-dind)
if [ -n "$HTTP_PROXY" ]; then
  echo "foo"
  mkdir -p "$HOME"/.docker/ && echo "{ \"proxies\": { \"default\": { \"httpProxy\": \"$HTTP_PROXY\", \"httpsProxy\": \"$HTTP_PROXY\" } }}" > "$HOME"/.docker/config.json
fi

