FROM ubuntu:22.04

RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends tzdata ca-certificates iproute2 bind9-dnsutils iputils-ping curl python3 python3-requests iptables wireguard-tools jq tcpdump python3-flask

RUN curl --verbose https://raw.githubusercontent.com/gdraheim/docker-systemctl-replacement/b18d67e521f0d1cf1d705dbb8e0416bef23e377c/files/docker/systemctl3.py --output /usr/bin/systemctl
RUN echo "01beb201d2045c5e548d012bde9b6ae6113392a57bbea7b3e81131aac995f77a /usr/bin/systemctl" | sha256sum --check --status
RUN chmod +x /usr/bin/systemctl

WORKDIR /opt
COPY ./.ci/docker/edgar/scripts/* /opt

