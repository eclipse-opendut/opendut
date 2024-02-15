#!/usr/bin/env bash
set -e
set -x

# persist edgar container logs
TARGET_DIR="${1:-docker-logs}"

if [ -d "$TARGET_DIR" ]; then
  echo "Directory $TARGET_DIR already exists. Exiting."
  exit 1
else
  mkdir -p "$TARGET_DIR"
fi

DOCKER_CONTAINERS="$(docker ps --all --format='{{.Names}}')"
for DOCKER_NAME in $DOCKER_CONTAINERS; do
  echo "Logs for $DOCKER_NAME"
  docker logs "$DOCKER_NAME" > ./docker-logs/docker-"$DOCKER_NAME".log 2>&1
done

# persist edgar logs
EDGAR_CONTAINERS="$(docker ps --all --format='{{.Names}}' --filter "name=edgar-*")"
for EDGAR_NAME in $EDGAR_CONTAINERS; do
  echo "Logs for $EDGAR_NAME"
  mkdir ./docker-logs/"$EDGAR_NAME"
  docker cp "$EDGAR_NAME":/logs/ ./docker-logs/"$EDGAR_NAME"/
done
