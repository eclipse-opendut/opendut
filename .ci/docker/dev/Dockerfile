FROM ubuntu:22.04 AS builder

# Add Docker's official GPG key:
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates curl gnupg && \
    install -m 0755 -d /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg && \
    chmod a+r /etc/apt/keyrings/docker.gpg

# Add the repository to Apt sources:
RUN echo \
  "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
RUN apt-get update && apt-get install -y --no-install-recommends docker-ce-cli docker-compose-plugin
RUN dpkg -L docker-compose-plugin

FROM ubuntu:22.04
ARG PUID=1000
ARG PGID=1000
ARG PUSER
ARG PGROUP
ARG DOCKER_GID
ARG OPENDUT_REPO_ROOT

RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends tzdata ca-certificates \
        build-essential \
        binutils-arm-linux-gnueabihf \
        lib32z1-dev \
        gcc-arm-linux-gnueabihf \
        curl \
        git \
        sudo \
        uuid \
        iproute2 bind9-dnsutils iputils-ping curl iptables wireguard-tools jq tcpdump

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

COPY --from=builder /usr/bin/docker /usr/bin/docker
COPY --from=builder /usr/libexec/docker /usr/libexec/docker

WORKDIR /src

# create group and user as specified in build args (same as host of developer)
RUN groupadd -g ${PGID} ${PGROUP}
RUN useradd -m -u ${PUID} -g ${PGID} -o -s /bin/bash ${PUSER}

# add docker group in container
RUN groupadd -g ${DOCKER_GID} docker
# allow users to run docker in docker
RUN usermod -a -G docker ${PUSER} && newgrp docker


RUN cp -r /root/.cargo /home/${PUSER}/ && chown -R ${PUSER}:${PGROUP} /home/${PUSER}/.cargo
RUN echo 'source /home/${PUSER}/.cargo/env' >> /home/${PUSER}/.bashrc

# allow changing certificates
RUN chown -R ${PUSER}:${PGROUP} /usr/local/share/ca-certificates/
RUN echo "${PUSER} ALL=(ALL:ALL) NOPASSWD: /usr/sbin/update-ca-certificates" >> /etc/sudoers

# allow appending hosts in /etc/hosts as ordinary user in a coordinated manner at runtime
RUN echo "${PUSER} ALL=(ALL:ALL) NOPASSWD: /usr/sbin/append_hosts.sh" >> /etc/sudoers
COPY ./.ci/docker/dev/append_hosts.sh /usr/sbin/append_hosts.sh

# allow all sudo commands
#RUN echo "${PUSER} ALL=(ALL:ALL) NOPASSWD: ALL" >> /etc/sudoers

# Set up OpenDUT repo path to ensure docker in docker cross compilation will use the same path as the host
RUN mkdir -p $OPENDUT_REPO_ROOT
WORKDIR $OPENDUT_REPO_ROOT

RUN echo shopt -s histappend >> /home/$PUSER/.bashrc
RUN echo "PROMPT_COMMAND=\"history -a;\$PROMPT_COMMAND\"" >> /home/$PUSER/.bashrc

COPY ./.ci/docker/dev/entrypoint.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
