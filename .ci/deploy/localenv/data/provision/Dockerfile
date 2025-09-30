FROM ubuntu:24.04
LABEL authors="av"

RUN userdel -r ubuntu
RUN groupadd --gid 1000 carl
RUN useradd --create-home --uid 1000 --gid carl --shell /bin/bash carl

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends pwgen openssl rsync && rm -rf /var/lib/apt/lists/*
COPY .ci/deploy/localenv/data/provision/entrypoint.sh /entrypoint.sh

RUN mkdir /scripts
COPY .ci/deploy/localenv/data/provision/scripts/*.sh /scripts

RUN chmod +x /entrypoint.sh /scripts/*.sh

ENTRYPOINT ["/entrypoint.sh"]