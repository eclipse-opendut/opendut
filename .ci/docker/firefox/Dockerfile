FROM ubuntu:22.04 as builder
RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends libnss3-tools

# version of firefox from here: https://github.com/linuxserver/docker-firefox/releases
# added a copy to ghcr due to connectivity issues with the original source
FROM ghcr.io/eclipse-opendut/firefox:124.0.1-r0-ls156

COPY --from=builder /usr/bin/certutil /usr/bin/certutil

COPY ./.ci/docker/firefox/cont-init.d/ /custom-cont-init.d
COPY ./.ci/docker/firefox/custom-services.d/ /custom-services.d
