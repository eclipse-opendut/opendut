#!/bin/bash
set -e
set -x


/scripts/generate-ca.sh
/scripts/generate-certificate.sh opendut.local
/scripts/generate-certificate.sh auth.opendut.local
/scripts/generate-certificate.sh netbird.opendut.local
/scripts/generate-certificate.sh netbird-api.opendut.local
/scripts/generate-certificate.sh signal.opendut.local
/scripts/generate-certificate.sh carl.opendut.local
/scripts/generate-certificate.sh nginx-webdav.opendut.local
/scripts/generate-certificate.sh opentelemetry.opendut.local
/scripts/generate-certificate.sh monitoring.opendut.local
