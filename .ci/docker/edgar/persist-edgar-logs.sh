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

# collect a list of all containers
docker ps --all > ./docker-logs/docker-ps-all.log 2>&1

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

# collect facts
for EDGAR_NAME in $EDGAR_CONTAINERS; do
  set +e  # do not exit on error

  # persist ip address configuration
  docker exec "$EDGAR_NAME" ip address show > ./docker-logs/"$EDGAR_NAME"/ip-address-show.log 2>&1
  # running processes
  docker exec "$EDGAR_NAME" ps axu > ./docker-logs/"$EDGAR_NAME"/processes-ps-axu.log 2>&1
  # netbird status
  docker exec "$EDGAR_NAME" /opt/opendut/edgar/netbird/netbird status > ./docker-logs/"$EDGAR_NAME"/netbird-status.log 2>&1
  docker exec "$EDGAR_NAME" /opt/opendut/edgar/netbird/netbird status --detail > ./docker-logs/"$EDGAR_NAME"/netbird-status-detail.log 2>&1
  # persist wireguard peer connection details
  docker exec "$EDGAR_NAME" wg > ./docker-logs/"$EDGAR_NAME"/wg.log 2>&1
done
