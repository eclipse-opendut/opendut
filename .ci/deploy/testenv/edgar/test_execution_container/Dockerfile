FROM ubuntu:24.04

RUN apt-get update && apt-get install -y nmap

COPY ./entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
