#!/usr/bin/env bash
set -e  # exit on error
set -x

# persist edgar container logs
LOG_DIRECTORY="${1:-./docker-logs}"

if [ -d "$LOG_DIRECTORY" ]; then
  echo "Directory $LOG_DIRECTORY already exists. Exiting."
  exit 1
else
  mkdir -p "$LOG_DIRECTORY"
fi

#################################################
# Collect logs of all containers
#################################################
docker ps --all > "$LOG_DIRECTORY"/docker-ps-all.log 2>&1

# ignore telemetry containers: otel-collector, opendut-traefik, opendut-loki, opendut-netbird-coturn, opendut-grafana
OPENDUT_DOCKER_CONTAINERS="$(docker ps --all --format='{{.Names}}' --filter "name=opendut-*" | grep -vE 'otel-collector|opendut-traefik|opendut-loki|opendut-netbird-coturn|opendut-grafana' || [[ $? == 1 ]])"
for DOCKER_NAME in $OPENDUT_DOCKER_CONTAINERS; do
  echo "Logs for $DOCKER_NAME"
  docker logs "$DOCKER_NAME" > "$LOG_DIRECTORY"/docker-"$DOCKER_NAME".log 2>&1
done

#################################################
# Collect EDGAR logs (netbird client, EDGAR service logs)
#################################################
OPENDUT_EDGAR_CONTAINERS="$(docker ps --all --format='{{.Names}}' --filter "name=edgar-*")"
for EDGAR_NAME in $OPENDUT_EDGAR_CONTAINERS; do
  echo "Logs for $EDGAR_NAME"
  mkdir -p "$LOG_DIRECTORY"/"$EDGAR_NAME"
  docker cp "$EDGAR_NAME":/logs/ "$LOG_DIRECTORY"/"$EDGAR_NAME"/
done

#################################################
# Collect other facts
#################################################
for EDGAR_NAME in $OPENDUT_EDGAR_CONTAINERS; do
  # persist ip address configuration
  docker exec "$EDGAR_NAME" ip address show > "$LOG_DIRECTORY/$EDGAR_NAME/ip-address-show.log" 2>&1
  # running processes
  docker exec "$EDGAR_NAME" ps axu > "$LOG_DIRECTORY/$EDGAR_NAME/processes-ps-axu.log" 2>&1
  # NetBird status
  docker exec "$EDGAR_NAME" /opt/opendut/edgar/netbird/netbird status > "$LOG_DIRECTORY/$EDGAR_NAME/netbird-status.log" 2>&1
  docker exec "$EDGAR_NAME" /opt/opendut/edgar/netbird/netbird status --detail > "$LOG_DIRECTORY/$EDGAR_NAME/netbird-status-detail.log" 2>&1
  # persist WireGuard peer connection details
  docker exec "$EDGAR_NAME" wg > "$LOG_DIRECTORY/$EDGAR_NAME/wg.log" 2>&1
done

#################################################
# Collect details about CARL
# using opendut-cleo container of the localenv environment
#################################################
if docker ps -a --format '{{.Names}}' --filter "name=opendut-cleo" | grep -q "^opendut-cleo" || false; then
  mkdir -p "$LOG_DIRECTORY"/cleo/
  CLEO_SUBCOMMANDS="cluster-descriptors cluster-deployments peers devices"
  for COMMAND in $CLEO_SUBCOMMANDS
  do
    docker exec opendut-cleo opendut-cleo list --output json "$COMMAND" > "$LOG_DIRECTORY"/cleo/opendut-cleo-"$COMMAND".json 2> "$LOG_DIRECTORY"/cleo/opendut-cleo-"$COMMAND".error.log
  done

  # Collect peer information
  if KNOWN_PEERS=$(docker exec opendut-cleo opendut-cleo list --output json peers | jq -r '.[].id'); then
    for PEER in $KNOWN_PEERS
    do
      docker exec opendut-cleo opendut-cleo describe --output json peer "$PEER" > "$LOG_DIRECTORY"/cleo/opendut-cleo-peer-"$PEER".json 2> "$LOG_DIRECTORY"/cleo/opendut-cleo-peer-"$PEER".error.log
    done
  fi
else
  echo "opendut-cleo container not found, skipping CARL logs collection"
fi