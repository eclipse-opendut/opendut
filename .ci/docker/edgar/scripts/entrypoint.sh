#!/bin/bash
set -e  # exit on error
set -x  # print commands

source "$(dirname "$0")/functions.sh"


main() {
  OPENDUT_REQUIRE_DIST="${1:-true}"

  copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA1"
  copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA2"
  append_data_from_env_variable OPENDUT_HOSTS /etc/hosts

  export OPENDUT_ARTIFACTS_DIR=/opt/artifacts
  ls -la $OPENDUT_ARTIFACTS_DIR

  # unpack distribution archives
  extract_opendut_artifact opendut-edgar "$OPENDUT_REQUIRE_DIST"
  extract_opendut_artifact opendut-cleo "$OPENDUT_REQUIRE_DIST"

}


# symlink netbird to known binary path
#OPENDUT_EDGAR_NETBIRD_BINARY=/opt/opendut-network/edgar/netbird/netbird
#if [ -e "$OPENDUT_EDGAR_NETBIRD_BINARY" ]; then
#  ln -sf /opt/opendut-network/edgar/netbird/netbird /usr/local/bin/distribution/netbird
#else
#  echo "Could not find netbird binary in $OPENDUT_EDGAR_NETBIRD_BINARY."
#  exit 1
#fi

if [ -n "$1" ]; then
  if [ "$1" == "bash" ]; then
    main "${OPENDUT_REQUIRE_DIST:-false}"
  else
    # run by ci, requires artifacts
    main "true"
  fi

  exec "$@"

else
  main "${OPENDUT_REQUIRE_DIST:-false}"
fi
