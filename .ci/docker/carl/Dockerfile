FROM ubuntu:22.04
ARG VERSION

RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends tzdata ca-certificates curl && rm -rf /var/lib/apt/lists/*

WORKDIR /opt
ADD target/ci/distribution/x86_64-unknown-linux-gnu/opendut-carl-linux-x86_64-$VERSION.tar.gz /opt

EXPOSE 8080

COPY ./.ci/docker/carl/entrypoint.sh /opt/entrypoint.sh

RUN groupadd --gid 1000 carl
RUN useradd --create-home --uid 1000 --gid carl --shell /bin/bash carl

USER carl

ENTRYPOINT ["/opt/entrypoint.sh"]
CMD ["/opt/opendut-carl/opendut-carl"]
