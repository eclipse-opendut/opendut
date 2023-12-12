# Edgar startup in docker compose

Example setup with multiple edgar peers in docker.

## Prepare

### Environment variables

```
NETBIRD_SETUP_KEY=
NETBIRD_GROUP=
OPENDUT_EDGAR_REPLICAS=4
NETBIRD_API_TOKEN=
NETBIRD_MANAGEMENT_API=https://netbird-api.opendut.local
# example usage of custom CA1 CA2
OPENDUT_CUSTOM_CA1=""
OPENDUT_CUSTOM_CA2=""
OPENDUT_HOSTS=""
```

### Create one-line version of /etc/hosts in OPENDUT_HOSTS environment variable

Use the following to create OPENDUT_HOSTS variable or the awk command below.
```
export NETBIRD_IP=1.2.3.4
echo "$NETBIRD_IP  opendut.local\n$NETBIRD_IP  auth.opendut.local\n$NETBIRD_IP  netbird.opendut.local\n$NETBIRD_IP  netbird-api.opendut.local\n$NETBIRD_IP  signal.opendut.local\n"
```


### Create one-line representation of a file to pass as environment variable

This may be used to pass entire file to the container, e.g.:
- Certificate to be written
- Additional hosts for /etc/hosts

Following command prints the content of the file with newlines replaced by \n and \r removed.
```
awk 'NF {sub(/\r/, ""); printf "%s\\n",$0;}' resources/development/tls/insecure-development-ca.pem
```

## GitHub Runner workflow

```
docker compose build
docker compose up -d
docker exec -ti edgar_router /opt/wait_until_ready.sh
docker exec -ti edgar-peer-1 /opt/wait_until_ready.sh
docker exec -ti edgar-peer-1 /opt/pingall.sh
docker exec -ti edgar-peer-1 python3 /opt/delete_peers.py
docker compose down

```

