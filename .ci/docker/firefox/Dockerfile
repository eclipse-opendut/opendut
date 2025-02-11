FROM ubuntu:24.04 AS builder
RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends libnss3-tools

# version of firefox from here: https://github.com/linuxserver/docker-firefox/releases
# added a copy to ghcr due to connectivity issues with the original source
FROM ghcr.io/eclipse-opendut/firefox:135.0build2-0ubuntu0.24.04.1mt2-ls218

COPY --from=builder /usr/bin/certutil /usr/bin/certutil

COPY ./.ci/docker/firefox/cont-init.d/ /custom-cont-init.d
COPY ./.ci/docker/firefox/custom-services.d/ /custom-services.d
