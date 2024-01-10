FROM ubuntu:22.04 as builder
RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends libnss3-tools

FROM lscr.io/linuxserver/firefox:119.0-r0-ls131

COPY --from=builder /usr/bin/certutil /usr/bin/certutil

COPY ./.ci/docker/firefox/entrypoint.sh /entrypoint.sh
COPY ./.ci/docker/firefox/postinit.sh /postinit.sh
COPY ./.ci/docker/firefox/postinit-register.sh /etc/cont-init.d/run-firefox-postinit

ENTRYPOINT ["/entrypoint.sh"]
