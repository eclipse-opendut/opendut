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
	OPENDUT_DOMAIN_NETBIRD_RELAY
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

# Create intermediate certificate authority
/scripts/generate-intermediate-ca.sh "intermediate-ca"

# Create server certificates
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_SUFFIX}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_AUTH}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NETBIRD}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NETBIRD_RELAY}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NETBIRD_API}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_SIGNAL}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_CARL}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NGINX_WEBDAV}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_OPENTELEMETRY}"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_MONITORING}"

# Create a client certificate for authentication purposes
/scripts/generate-client-certificate.sh "opendut-auth-client"
/scripts/generate-client-certificate.sh "opendut-browser-client"
/scripts/generate-client-certificate.sh "edgar-leader"

# Create CARL certificate from intermediate CA (for testing) and store in different directory (/provision/pki/deploy-intermediate)
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_SUFFIX}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_AUTH}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NETBIRD}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NETBIRD_RELAY}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NETBIRD_API}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_SIGNAL}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_CARL}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_NGINX_WEBDAV}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_OPENTELEMETRY}" "intermediate-ca" "deploy-intermediate"
/scripts/generate-certificate.sh "${OPENDUT_DOMAIN_MONITORING}" "intermediate-ca" "deploy-intermediate"

# Create a client certificate derived from intermediate CA
/scripts/generate-client-certificate.sh "client-intermediate-test" "intermediate-ca"
