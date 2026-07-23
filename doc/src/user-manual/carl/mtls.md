# Mutual TLS (mTLS) Configuration

By default, CARL uses TLS to encrypt connections but does not require clients to present certificates. When mTLS is enabled, CARL additionally verifies that connecting clients (EDGAR, CLEO, LEA via Traefik) present a valid client certificate signed by a trusted CA.

## Configuration

### Enabling mTLS on CARL

Set the following configuration values (shown as environment variables):

```bash
# Enable mTLS - CARL will require client certificates
OPENDUT_CARL_NETWORK_TLS_SERVER_AUTH_ENABLED=true

# CA certificate used to verify client certificates
# All client certs must be signed by this CA (or a CA chained to it)
OPENDUT_CARL_NETWORK_TLS_SERVER_AUTH_CA=/path/to/ca.pem
```

Or in `carl.toml`:

```toml
[network.tls.server.auth]
enabled = true
ca = "/path/to/ca.pem"
```

### Using an External CA

When using your own CA (e.g., a corporate CA or a public CA like Let's Encrypt) instead of the auto-generated development CA, you need to configure:

1. **CARL's trusted CA for client verification** — tells CARL which CA to trust when verifying incoming client certificates:
   ```bash
   OPENDUT_CARL_NETWORK_TLS_SERVER_AUTH_CA=/path/to/your-ca.pem
   ```

2. **CARL's server certificate** — CARL's own identity, must be valid for CARL's hostname:
   ```bash
   OPENDUT_CARL_NETWORK_TLS_CERTIFICATE=/path/to/carl-server.pem
   OPENDUT_CARL_NETWORK_TLS_KEY=/path/to/carl-server.key
   ```

3. **CARL's client certificate** — used by CARL when connecting to other services (Keycloak, OpenTelemetry) that also require mTLS:
   ```bash
   OPENDUT_CARL_NETWORK_TLS_CLIENT_AUTH_ENABLED=true
   OPENDUT_CARL_NETWORK_TLS_CLIENT_AUTH_CERTIFICATE=/path/to/carl-client.pem
   OPENDUT_CARL_NETWORK_TLS_CLIENT_AUTH_KEY=/path/to/carl-client.key
   ```

When mTLS is enabled, all clients (EDGAR, CLEO) must present a valid client certificate. See the [EDGAR mTLS setup guide](../edgar/setup.md#mtls-client-authentication) for client-side configuration.

## Healthcheck

When mTLS is enabled, the Docker healthcheck for CARL must also present a client certificate. The localenv docker-compose override handles this automatically by passing CARL's client certificate into the healthcheck curl command.

For custom deployments (e.g., Kubernetes), ensure that liveness/readiness probes either:
- Use an `exec` probe with `curl --cert <client-cert> --key <client-key>`
- Or expose a separate non-mTLS health endpoint on a different port

## Example: Complete mTLS Configuration

```toml
[network.tls]
enabled = true
certificate = "/etc/opendut/tls/carl-server.pem"
key = "/etc/opendut/tls/carl-server.key"
ca = "/etc/opendut/tls/your-ca.pem"

[network.tls.client.auth]
enabled = true
certificate = "/etc/opendut/tls/carl-client.pem"
key = "/etc/opendut/tls/carl-client.key"

[network.tls.server.auth]
enabled = true
ca = "/etc/opendut/tls/your-ca.pem"
```
