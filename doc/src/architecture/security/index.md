# Security

This chapter outlines the security architecture of OpenDuT, detailing how integrity, confidentiality, authentication, and authorization are enforced across the system.
It describes the mechanisms used to protect sensitive data and communications, explains the integration with identity providers, and presents the threat model that guides the implementation of security controls.
The goal is to provide a comprehensive overview of how OpenDuT safeguards its assets and mitigates potential risks.

## Integrity and Confidentiality

All service communications are encrypted using TLS to ensure data integrity and confidentiality.
Peer-to-peer communication within clusters is secured via [WireGuard](https://www.wireguard.com/) which is managed with the help of [NetBird](https://docs.netbird.io/about-netbird/why-wireguard-with-netbird).
OpenDuT EDGAR uses WireGuard tunnels to encapsulate the Ethernet and CAN traffic, ensuring secure routing between devices.


## Authentication and Authorization

OpenDuT uses OAuth 2.0 for user authentication and resource authorization, with Keycloak serving as the central authorization server.
OpenDuT relies on Keycloak for managing OpenID Connect (OIDC) clients used by technical components.
Each EDGAR instance requires a dedicated client credential in Keycloak.
CARL is responsible for creating these clients, which necessitates elevated permissions within the Keycloak realm.
This approach enables integration with upstream identity providers but tightly couples the system to Keycloak, making replacement with alternative solutions challenging.
Integration with an upstream identity provider means that Keycloak can delegate authentication to an external system (such as LDAP, Active Directory, or another OAuth/OIDC provider).
Users authenticate with the upstream provider, and Keycloak acts as a broker, passing identity information to OpenDuT.
This allows centralized user management and single sign-on across multiple systems, leveraging existing identity infrastructure.

### OIDC Authentication Flows
The following OpenID Connect (OIDC) flows are implemented:

* Authorization Code Flow with PKCE: Used by web applications (e.g., OpenDuT-LEA) via a public client in Keycloak.
* Client Credentials Grant: Used by confidential clients:
  * OpenDuT-CARL
  * OpenDuT-EDGAR
  * OpenDuT-CLEO

### Client applications

* OpenDuT-LEA (public client)
* OpenDuT-CLEO (confidential client)
* OpenDuT-EDGAR (confidential client)
* OpenDuT-CARL (confidential client)
* NetBird client: 
  * setup key or device code flow during registration
  * custom authentication for regular operation
* OpenDuT-CARL is a client:
  * to the Keycloak REST API (confidential client)
  * to the NetBird Management API (bearer token obtained via client credentials grant)

### Resource server

* OpenDuT-CARL: 
  * used by LEA and CLEO to manage peers and clusters
  * used by EDGAR to report status and manage connected devices
* NetBird Management:
  * used by NetBird clients in EDGAR
  * used by CARL to manage peers
* OpenTelemetry Collector (used by OpenDuT components for telemetry export)
* Keycloak Rest API:
  * used by OpenDuT-CARL to register(create) and delete OAuth clients

### Typical User Interaction

A typical workflow:
* User authenticates in LEA.
* Creates a new peer.
* Generates a setup string containing confidential connection information (client ID and secret).
* Transfers the setup string securely to the peer.
* Uses the setup string to configure an EDGAR instance on a peer.


## Threat model

OpenDuT follows a zero-trust security model, assuming that no component is inherently trustworthy.
All interactions between components are authenticated and authorized, and all data is encrypted in transit.

* Assets: Sensitive data (setup strings, credentials), ECUs, user accounts, network traffic.
* Threat Actors: External attackers, malicious insiders, compromised peers, supply chain risks.
* Attack Vectors: Network interception, unauthorized access, privilege escalation, code injection, misconfiguration, vulnerabilities in dependencies.
* Mitigations:
    * Enforce TLS and VPN for all communications.
    * Use strong authentication and authorization (OAuth 2.0, OIDC).
    * Limit privileges with role based access control (RBAC).
    * Regularly update and patch dependencies.
    * Monitor and log security events.
    * Secure storage and transmission of secrets.

