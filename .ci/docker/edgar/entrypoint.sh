#!/bin/sh

if [ -n "$OPENDUT_EDGAR_SETUP_STRING" ]; then
  /opt/opendut-edgar/opendut-edgar setup managed --no-confirm --skip-service-run --log-file=-
else
  echo "Environment variable 'OPENDUT_EDGAR_SETUP_STRING' not specified. Skipping EDGAR Setup."
fi

/opt/opendut-edgar/opendut-edgar service
