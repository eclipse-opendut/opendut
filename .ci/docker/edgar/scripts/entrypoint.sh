#!/bin/bash
set -e  # exit on error
set -x  # print commands

source "$(dirname "$0")/functions.sh"

copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA1"
copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA2"
append_data_from_env_variable OPENDUT_HOSTS /etc/hosts

export OPENDUT_ARTIFACTS_DIR=/opt/artifacts
ls -la $OPENDUT_ARTIFACTS_DIR

extract_opendut_artifact() {
  NAME="$1"
  REQUIRED="true"

  OPENDUT_DIST_ARCHIVE_PATH=$(find $OPENDUT_ARTIFACTS_DIR -name "$NAME-x86_64-unknown-linux-gnu*.tar.gz" -print0 | head)
  if [ -e "$OPENDUT_DIST_ARCHIVE_PATH" ]; then
    tar xf "$OPENDUT_DIST_ARCHIVE_PATH"
    OPENDUT_ARTIFACT_BINARY="/opt/$NAME/$NAME"
    if [ -e "$OPENDUT_ARTIFACT_BINARY" ]; then
      ln -sf /opt/"$NAME"/"$NAME" /usr/local/opendut/bin/distribution/"$NAME"
    else
      echo "Could not extract artifact binary \'$OPENDUT_ARTIFACT_BINARY\' from archive \'$OPENDUT_DIST_ARCHIVE_PATH\'."
      exit 1
    fi
  else
    echo "Could not find distribution archive for \'$NAME\'."
    if [ "$REQUIRED" = true ] ; then
      exit 1
    fi
  fi
}

# unpack distribution archives
extract_opendut_artifact opendut-edgar "true"
extract_opendut_artifact opendut-cleo "true"

# symlink netbird to known binary path
#OPENDUT_EDGAR_NETBIRD_BINARY=/opt/opendut-network/edgar/netbird/netbird
#if [ -e "$OPENDUT_EDGAR_NETBIRD_BINARY" ]; then
#  ln -sf /opt/opendut-network/edgar/netbird/netbird /usr/local/bin/distribution/netbird
#else
#  echo "Could not find netbird binary in $OPENDUT_EDGAR_NETBIRD_BINARY."
#  exit 1
#fi


exec "$@"