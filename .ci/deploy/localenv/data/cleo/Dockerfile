FROM ubuntu:24.04

RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends tzdata ca-certificates curl jq && rm -rf /var/lib/apt/lists/*

WORKDIR /opt

COPY ./.ci/deploy/localenv/data/cleo/entrypoint.sh /opt/entrypoint.sh

RUN chmod +x /opt/entrypoint.sh

ENTRYPOINT ["/opt/entrypoint.sh"]
