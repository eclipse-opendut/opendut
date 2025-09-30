FROM ubuntu:24.04
ARG VERSION
ENV TARGET_TRIPLE=x86_64-unknown-linux-gnu

RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends tzdata ca-certificates curl jq && rm -rf /var/lib/apt/lists/*

WORKDIR /opt
ADD target/ci/distribution/x86_64-unknown-linux-gnu/opendut-carl-$TARGET_TRIPLE-$VERSION.tar.gz /opt

EXPOSE 8080

COPY ./.ci/docker/carl/entrypoint.sh /opt/entrypoint.sh

RUN userdel -r ubuntu
RUN groupadd --gid 1000 carl
RUN useradd --create-home --uid 1000 --gid carl --shell /bin/bash carl
RUN mkdir -p /etc/opendut/tls && chown -R carl:carl /etc/opendut
RUN mkdir -p /opt/opendut-carl/config/ && chown -R carl:carl /opt/opendut-carl/
RUN mkdir -p /var/lib/opendut/carl/ && chown -R carl:carl /var/lib/opendut/carl/
USER carl

VOLUME /opt/opendut-carl/config
VOLUME /var/lib/opendut/carl/

ENTRYPOINT ["/opt/entrypoint.sh"]
CMD ["/opt/opendut-carl/opendut-carl"]
