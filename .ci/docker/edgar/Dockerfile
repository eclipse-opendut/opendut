FROM ubuntu:22.04 as cannelloni-builder
RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    ca-certificates wget build-essential cmake libsctp-dev

RUN mkdir /tmp/cannelloni
WORKDIR /tmp/cannelloni
RUN wget https://github.com/mguentner/cannelloni/archive/refs/tags/v1.1.0.tar.gz -O cannelloni-1.1.0.tar.gz 
RUN echo "0dcb9277b21f916f5646574b9b2229d3b8e97d5e99b935a4d0b7509a5f0ccdcd cannelloni-1.1.0.tar.gz" | sha256sum --check --status
RUN tar --strip-components=1 -xvf cannelloni-1.1.0.tar.gz 
RUN cmake -DCMAKE_BUILD_TYPE=Release && make

FROM cruizba/ubuntu-dind:jammy-26.0.1

RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    tzdata ca-certificates \
    iproute2 bind9-dnsutils iputils-ping fping iptables wireguard-tools tcpdump netcat \
    curl jq uuid-runtime \
    python3 python3-requests python3-flask \
    python3-can can-utils libsctp1

COPY --from=cannelloni-builder /tmp/cannelloni/cannelloni /usr/local/bin/cannelloni
COPY --from=cannelloni-builder /tmp/cannelloni/libcannelloni-common.so.0.0.1 /usr/local/lib/libcannelloni-common.so.0

ENV LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"

RUN curl --verbose https://raw.githubusercontent.com/gdraheim/docker-systemctl-replacement/b18d67e521f0d1cf1d705dbb8e0416bef23e377c/files/docker/systemctl3.py --output /usr/bin/systemctl
RUN echo "01beb201d2045c5e548d012bde9b6ae6113392a57bbea7b3e81131aac995f77a /usr/bin/systemctl" | sha256sum --check --status
RUN chmod +x /usr/bin/systemctl

COPY ./.ci/docker/edgar/test_execution_container /opt/test_execution_container

WORKDIR /opt
RUN mkdir /opt/signal/ /logs/
COPY ./.ci/docker/edgar/scripts/* /opt
COPY ./.ci/docker/edgar/configurations/ /opt/configurations/
RUN /opt/dind_proxy.sh

RUN mkdir -p /usr/local/opendut/bin/distribution /usr/local/opendut/bin/debug /opt/opendut/edgar/netbird/
ENV PATH="${PATH}:/usr/local/opendut/bin/distribution/:/usr/local/opendut/bin/debug/:/opt/opendut/edgar/netbird/"

ENTRYPOINT ["/opt/entrypoint.sh"]
