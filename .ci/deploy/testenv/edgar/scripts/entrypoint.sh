#!/bin/bash
set -e  # exit on error
set -x  # print commands

source "$(dirname "$0")/functions.sh"


main() {
  OPENDUT_REQUIRE_DIST="${1:-true}"

  export OPENDUT_ARTIFACTS_DIR=/opt/artifacts
  ls -la $OPENDUT_ARTIFACTS_DIR

  # unpack distribution archives
  extract_opendut_artifact opendut-edgar "$OPENDUT_REQUIRE_DIST"
  extract_opendut_artifact opendut-cleo "$OPENDUT_REQUIRE_DIST"

}

# update certificates
echo "Updating CA certificates"
update-ca-certificates

# configure http proxy for Docker in Docker container (cruizba/ubuntu-dind)
/opt/dind_proxy.sh


if [ -n "$1" ]; then
  if [ "$1" == "bash" ]; then
    # do not extract artifacts when run by `cargo theo dev edgar-shell`
    main "${OPENDUT_REQUIRE_DIST:-false}"
  else
    # run by ci, requires artifacts
    main "true"
  fi

  exec "$@"

else
  main "${OPENDUT_REQUIRE_DIST:-false}"
fi
