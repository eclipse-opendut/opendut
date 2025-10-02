FROM ubuntu:24.04

ARG VERSION
ARG cannelloni_version=1.1.0
ENV TARGET_TRIPLE=x86_64-unknown-linux-gnu

# Install OS package dependencies. We need:
#   - `ca-certificates` for `update-ca-certificates` command used during EDGAR Setup
#   - `python3` to run systemctl-replacement script
#   - `can-utils` for `cangw` used by EDGAR Service
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
    ca-certificates python3 can-utils && \
    rm -rf /var/lib/apt/lists/*


# Install systemctl drop-in replacement
ADD --checksum=sha256:01beb201d2045c5e548d012bde9b6ae6113392a57bbea7b3e81131aac995f77a \
    https://raw.githubusercontent.com/gdraheim/docker-systemctl-replacement/b18d67e521f0d1cf1d705dbb8e0416bef23e377c/files/docker/systemctl3.py \
    /usr/bin/systemctl
RUN chmod +x /usr/bin/systemctl


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

CMD ["/opt/opendut-edgar/opendut-edgar", "setup", "managed", "--no-confirm"]
