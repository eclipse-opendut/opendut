#!/bin/sh

if [ -z "${TURN_PASSWORD}" ]; then
  echo "Please provide a proper secret by overriding environment variable 'TURN_PASSWORD=<tbd>'!"
  exit 1
fi
if [ -z "$TURN_PORT" ]; then
  echo "Environment variable required: 'TURN_PORT=3478'. Port 3478 is default port."
  exit 1
fi
if [ -z "$TURN_USER" ]; then
  echo "Environment variable required: 'TURN_USER=self'."
  exit 1
fi

echo "Starting TURN/STUN server"
# --no-stdout-log
TURN_LOG_FILE=${TURN_LOG_FILE:-stdout}
TURN_TLS_PORT=${TURN_TLS_PORT:-5349}
# https://github.com/coturn/coturn/tree/master/docker/coturn#why-so-many-ports-opened
# see also RFC 5766. The default port range is 49152-65535.
TURN_PORT_START=${TURN_PORT_START:-49152}
TURN_PORT_END=${TURN_PORT_END:-65535}


exec turnserver -a -v -L 0.0.0.0 \
  --server-name "${TURN_DOMAIN}" \
  --realm="${TURN_REALM}" --listening-port "${TURN_PORT}" \
  --user="${TURN_USER}:${TURN_PASSWORD}" \
  --no-cli --no-software-attribute \
  --fingerprint --lt-cred-mech \
  --min-port "${TURN_PORT_START}" --max-port "${TURN_PORT_END}" \
  --log-file="${TURN_LOG_FILE}" --simple-log "${TURN_EXTRA_ARGS}"

# TODO: encrypt turn server communication
# --tls-listening-port="${TURN_TLS_PORT}" \
# --cert="${TURN_TLS_CERT}" --pkey="${TURN_TLS_PKEY}" \
