#!/bin/bash
set -e
set -x


/scripts/generate-ca.sh

# Fail fast if any required domain variable is empty
required_vars=(
    OPENDUT_DOMAIN_SUFFIX
	OPENDUT_DOMAIN_CARL
	OPENDUT_DOMAIN_AUTH
	OPENDUT_DOMAIN_NETBIRD
	OPENDUT_DOMAIN_NETBIRD_API
	OPENDUT_DOMAIN_SIGNAL
	OPENDUT_DOMAIN_NGINX_WEBDAV
	OPENDUT_DOMAIN_OPENTELEMETRY
	OPENDUT_DOMAIN_MONITORING
)
for v in "${required_vars[@]}"; do
	if [ -z "${!v:-}" ]; then
		echo "[recreate] ERROR: required env $v is not set" >&2
		exit 1
	fi
done

/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_SUFFIX}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_AUTH}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NETBIRD}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NETBIRD_API}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_SIGNAL}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_CARL}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NGINX_WEBDAV}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_OPENTELEMETRY}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_MONITORING}"