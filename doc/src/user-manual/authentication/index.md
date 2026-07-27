# Authentication

openDuT uses [Keycloak](https://www.keycloak.org/) as its central identity and access management system.
Keycloak handles authentication and authorization for all openDuT components — LEA, CLEO, EDGAR, and CARL — through standard [OAuth 2.0](https://oauth.net/2/) and [OpenID Connect (OIDC)](https://openid.net/connect/) protocols.

> **Why Keycloak?**  
> openDuT's security model requires a trusted identity broker that can manage both machine-to-machine credentials (used by CARL, EDGAR, and CLEO) and interactive user logins (used by LEA). Keycloak covers both use cases and additionally supports federation with upstream identity providers (such as LDAP, Active Directory, or corporate SSO systems), making it a natural fit for enterprise test environments.

This chapter is intended for **operators and administrators** who are responsible for deploying and securing openDuT. It covers:

- [Configuring an Identity Provider (IdP)](./identity-provider.md) — how to connect Keycloak to your organization's existing user directory or SSO system.
- [Role-based Access Control (RBAC)](./rbac.md) — how to restrict access to openDuT components so that only users holding a specific Keycloak role can connect.

## Relationship to the Architecture

The authentication concepts described here correspond to the [Security Architecture](../../architecture/security/index.md) chapter, which explains the overall OAuth 2.0 / OIDC design of the system at an architectural level.
This chapter takes the operator's perspective and focuses on *how to configure* Keycloak to achieve the desired access policies, rather than *why* the design is the way it is.

## Prerequisites

- A running openDuT deployment (see [CARL Setup](../carl/setup.md)).
- Administrative access to the Keycloak instance at `https://auth.<your-domain>/` (by default `https://auth.opendut.local/`).
- The `opendut` Keycloak realm has been created and seeded by the openDuT provisioning step.

> **Note:** All configuration steps in this chapter are performed inside the **`opendut` realm** of Keycloak, not in the `master` realm.

