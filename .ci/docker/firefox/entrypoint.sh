#!/bin/bash
set -e
set -x

# This is the first program launched at container start.
# We don't know where our binaries are and we cannot guarantee
# that the default PATH can access them.
# So this script needs to be entirely self-contained until it has
# at least /command, /usr/bin and /bin in its PATH.

copy_custom_certificate_from_environment_variable() {
  # takes a variable name and pipes the content to a certificate file
  custom_ca_variable_name="$1"
  if [ -n "${!custom_ca_variable_name}" ]; then
    echo -e "${!custom_ca_variable_name}" > /usr/local/share/ca-certificates/opendut_custom_ca_"${custom_ca_variable_name}".crt
    update-ca-certificates
  fi
}
append_data_from_env_variable() {
  var_name="$1"
  file_name="$2"
  if [ -n "${!var_name}" ]; then
    echo -e "${!var_name}" >> "$file_name"
  fi
}

die_with_error() {
        echo "terminating with error"
        exit 1
}
die_with_success() {
        echo "terminating"
        return 0
}

echo "Preparing ..."
copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA1"
copy_custom_certificate_from_environment_variable "OPENDUT_CUSTOM_CA2"
append_data_from_env_variable OPENDUT_HOSTS /etc/hosts


addpath () {
  x="$1"
  IFS=:
  set -- $PATH
  IFS=
  while test "$#" -gt 0 ; do
    if test "$1" = "$x" ; then
      return
    fi
    shift
  done
  PATH="${x}:$PATH"
}

if test -z "$PATH" ; then
  PATH=/bin
fi

addpath /bin
addpath /usr/bin
addpath /command
export PATH

# Now we're good: s6-overlay-suexec is accessible via PATH, as are
# all our binaries.
# Run preinit as root, then run stage0 as the container's user (can be
# root, can be a normal user).

exec s6-overlay-suexec \
  ' /package/admin/s6-overlay-3.1.5.0/libexec/preinit' \
  '' \
  /package/admin/s6-overlay-3.1.5.0/libexec/stage0 \
  "$@"
