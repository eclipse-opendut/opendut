FROM ubuntu:24.04

ARG VERSION
ARG cannelloni_version=1.1.0
ENV TARGET_TRIPLE=x86_64-unknown-linux-gnu

# Install OS package dependencies. We need:
#   - `ca-certificates` for `update-ca-certificates` command used during EDGAR Setup
#   - `can-utils` for `cangw` used by EDGAR Service
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
    ca-certificates can-utils && \
    rm -rf /var/lib/apt/lists/*


# Install cannelloni
RUN mkdir /tmp/cannelloni
WORKDIR /tmp/cannelloni
ADD --checksum=sha256:f98e4cad3a6d1df55423413ea52c676d2eb2ab3d783f25f720654f5797c83c44 \
    https://github.com/eclipse-opendut/cannelloni-build/releases/download/v${cannelloni_version}/cannelloni_manylinux_2_28-x64_${cannelloni_version}.tar.gz \
    /tmp/cannelloni/cannelloni.tar.gz
RUN tar --strip-components=1 -xvf cannelloni.tar.gz

RUN cp libcannelloni-common.so.0 /lib/
RUN cp libsctp.so* /lib/
RUN cp cannelloni /usr/local/bin/


# Bundle EDGAR
WORKDIR /opt
ADD target/ci/distribution/${TARGET_TRIPLE}/opendut-edgar-${TARGET_TRIPLE}-${VERSION}.tar.gz /opt

CMD /opt/opendut-edgar/opendut-edgar setup managed --no-confirm --skip-service-run --log-file=- && /opt/opendut-edgar/opendut-edgar service

# USER edgar  # TODO create unprivileged user
