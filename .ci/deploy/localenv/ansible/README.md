## How to use

1. Install Ansible on your machine, e.g. on Debian-based systems:
   ```sh
   sudo apt install ansible
   ```
2. Navigate into the Ansible directory:
   ```sh
   cd .ci/deploy/localenv/ansible/
   ```

3. Define an `inventory.yaml` with parameters for your hosts, for example like so:
   ```yaml
   backend:
     hosts:
      opendut1:
        opendut_version_ref: "development"
        opendut_carl_image_version: "0.8.0"
        opendut_compose_environment_config: |
          OPENDUT_DOMAIN_SUFFIX="opendut.local"
          OPENDUT_DOMAIN_NETBIRD="netbird.opendut.local"
          OPENDUT_DOMAIN_NETBIRD_API="netbird-api.opendut.local"
          OPENDUT_DOMAIN_AUTH="auth.opendut.local"
          OPENDUT_DOMAIN_SIGNAL="signal.opendut.local"
          OPENDUT_DOMAIN_OPENTELEMETRY="opentelemetry.opendut.local"
          OPENDUT_DOMAIN_NGINX_WEBDAV="nginx-webdav.opendut.local"
          OPENDUT_DOMAIN_MONITORING="monitoring.opendut.local"
          OPENDUT_DOMAIN_CARL="carl.opendut.local"
          SHARED_CERTS_HOST_DIR="/provision"
          SHARED_CERTS_MOUNT_DIR="/pki"
          SHARED_CERTS_UNENCRYPTED="/provision/pki/deploy"
          OPENDUT_CERT_CA_PATH="/provision/pki/opendut-ca.pem"
          OPENDUT_AUTH_NETWORK_TLS_KEY="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_AUTH}.key"
          OPENDUT_AUTH_NETWORK_TLS_CERTIFICATE="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_AUTH}.pem"
          OPENDUT_NETBIRD_NETWORK_TLS_CERTIFICATE="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_NETBIRD}.pem"
          OPENDUT_NETBIRD_NETWORK_TLS_KEY="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_NETBIRD}.key"
          OPENDUT_NETBIRD_API_NETWORK_TLS_CERTIFICATE="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_NETBIRD_API}.pem"
          OPENDUT_NETBIRD_API_NETWORK_TLS_KEY="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_NETBIRD_API}.key"
          OPENDUT_SIGNAL_NETWORK_TLS_CERTIFICATE="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_SIGNAL}.pem"
          OPENDUT_SIGNAL_NETWORK_TLS_KEY="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_SIGNAL}.key"
          OPENDUT_NGINX_WEBDAV_NETWORK_TLS_CERTIFICATE="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_NGINX_WEBDAV}.pem"
          OPENDUT_NGINX_WEBDAV_NETWORK_TLS_KEY="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_NGINX_WEBDAV}.key"
          OPENDUT_OPENTELEMETRY_NETWORK_TLS_CERTIFICATE="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_OPENTELEMETRY}.pem"
          OPENDUT_OPENTELEMETRY_NETWORK_TLS_KEY="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_OPENTELEMETRY}.key"
          OPENDUT_MONITORING_NETWORK_TLS_CERTIFICATE="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_MONITORING}.pem"
          OPENDUT_MONITORING_NETWORK_TLS_KEY="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_MONITORING}.key"
          OPENDUT_CARL_NETWORK_TLS_KEY="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_CARL}.key"
          OPENDUT_CARL_NETWORK_TLS_CERTIFICATE="${SHARED_CERTS_UNENCRYPTED}/${OPENDUT_DOMAIN_CARL}.pem"
      opendut-backend2:
        opendut_version_ref: "v0.1.2"
        ip_for_edge_hosts_file: "123.456.789.102"
        opendut_carl_image_version: "0.8.0"
        ## opendut_compose_environment_config: see example opendut1
     vars:
       repo_dir: "/data/opendut"
       backup_dir: "/data/backup/opendut/"
   edge:
     hosts:
       opendut-edge1:
         backend: opendut-backend1
         peer_id: "c1067a3a-6fd7-4466-96ef-56e1f51f778d"
       opendut-edge2:
         backend: opendut-backend1
         peer_id: "b4ade9ae-d2e4-46ac-84e5-2e7ef7aaca55"


   ```

4. Make sure you have entries in your SSH config for all the hosts declared in the `inventory.yaml`.

5. Run the scripts like so:
   ```sh
   ./playbook_backend.yaml -i inventory.yaml
   ./playbook_edge.yaml -i inventory.yaml
   ```
