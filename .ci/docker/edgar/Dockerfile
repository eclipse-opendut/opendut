FROM cruizba/ubuntu-dind:noble-28.1.1
ARG cannelloni_version=1.1.0

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    tzdata ca-certificates \
    iproute2 bind9-dnsutils iputils-ping fping iptables wireguard-tools tcpdump netcat-traditional \
    curl jq uuid-runtime less bridge-utils \
    python3 python3-requests python3-flask \
    python3-can can-utils libsctp1 vim


ENV LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"

RUN curl --verbose https://raw.githubusercontent.com/gdraheim/docker-systemctl-replacement/b18d67e521f0d1cf1d705dbb8e0416bef23e377c/files/docker/systemctl3.py --output /usr/bin/systemctl
RUN echo "01beb201d2045c5e548d012bde9b6ae6113392a57bbea7b3e81131aac995f77a /usr/bin/systemctl" | sha256sum --check --status
RUN chmod +x /usr/bin/systemctl

COPY ./.ci/docker/edgar/test_execution_container /opt/test_execution_container


RUN mkdir /tmp/cannelloni
WORKDIR /tmp/cannelloni
RUN wget https://github.com/eclipse-opendut/cannelloni-build/releases/download/v${cannelloni_version}/cannelloni_manylinux_2_28-x64_${cannelloni_version}.tar.gz -O /tmp/cannelloni/cannelloni.tar.gz
RUN echo "f98e4cad3a6d1df55423413ea52c676d2eb2ab3d783f25f720654f5797c83c44  cannelloni.tar.gz" | sha256sum --check --status
RUN tar --strip-components=1 -xvf cannelloni.tar.gz

RUN cp libcannelloni-common.so.0 /lib/
RUN cp libsctp.so* /lib/
RUN cp cannelloni /usr/local/bin/


WORKDIR /opt
RUN mkdir /opt/signal/ /logs/
COPY ./.ci/docker/edgar/scripts/* /opt
COPY ./.ci/docker/edgar/configurations/ /opt/configurations/
RUN /opt/dind_proxy.sh

RUN mkdir -p /usr/local/opendut/bin/distribution /usr/local/opendut/bin/debug /opt/opendut/edgar/netbird/
ENV PATH="${PATH}:/usr/local/opendut/bin/distribution/:/usr/local/opendut/bin/debug/:/opt/opendut/edgar/netbird/"

ENTRYPOINT ["/opt/entrypoint.sh"]
