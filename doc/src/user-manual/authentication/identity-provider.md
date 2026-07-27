# Configuring an Identity Provider in Keycloak

By default, openDuT uses Keycloak's built-in user database. In many organizations it is preferable — or mandatory — to let Keycloak delegate authentication to an existing **upstream identity provider (IdP)**, such as:

- A corporate **LDAP / Active Directory** directory service.
- An upstream **OIDC / OAuth 2.0** provider (e.g., Azure AD / Entra ID, Google Workspace, GitLab, GitHub).
- A **SAML 2.0** provider.

When an upstream IdP is configured, Keycloak acts as an identity *broker*: users authenticate against your existing provider, and Keycloak issues tokens that openDuT components consume. This keeps user management centralized in your organization's own directory while keeping the openDuT integration clean.

## Overview of the Federation Flow

```
User → LEA/CLEO → Keycloak (opendut realm) → Upstream IdP → Keycloak → openDuT component
```

1. The user initiates login in LEA (or CLEO).
2. Keycloak redirects the user to the upstream IdP's login page.
3. The user authenticates with their corporate credentials.
4. The upstream IdP redirects back to Keycloak with an assertion/token.
5. Keycloak maps the upstream identity to a local Keycloak user and issues an openDuT access token.
6. The openDuT component receives a token and grants access based on the user's roles.

---

## Connecting an OIDC / OAuth 2.0 Identity Provider

The following steps configure an upstream OIDC provider. The procedure for SAML is analogous — Keycloak's UI guides you through provider-specific fields.

### Step 1 — Register openDuT as a client in the upstream IdP

Before Keycloak can talk to your upstream IdP you must register it as a client/application there. Refer to your IdP's documentation. You will receive:

- A **Client ID**
- A **Client Secret**
- The **Issuer URL** (also called *Discovery URL* or *Well-Known URL*), e.g.:  
  `https://login.microsoftonline.com/<tenant-id>/v2.0`

Set the **Redirect / Callback URL** in the upstream IdP to:
```
https://auth.<your-domain>/realms/opendut/broker/<alias>/endpoint
```
Replace `<alias>` with the short name you will choose for this provider in the next step (e.g., `corporate-sso`).

### Step 2 — Add the Identity Provider in Keycloak

1. Log in to Keycloak at `https://auth.<your-domain>/` with administrator credentials.
2. Switch to the **`opendut`** realm using the realm selector in the top-left corner.
3. Navigate to **Identity Providers** in the left-hand menu.
4. Click **Add provider** and select **OpenID Connect v1.0** (or **SAML v2.0** for SAML).
5. Fill in the form:

   | Field | Value |
   |---|---|
   | **Alias** | A short unique name, e.g. `corporate-sso` |
   | **Display name** | Human-readable name shown on the login page, e.g. `Corporate SSO` |
   | **Discovery endpoint** | The upstream IdP's well-known URL, e.g. `https://login.example.com/.well-known/openid-configuration` |
   | **Client ID** | The Client ID from Step 1 |
   | **Client Secret** | The Client Secret from Step 1 |
   | **Default Scopes** | `openid profile email` (adjust as required by your IdP) |

6. Scroll down to **Advanced Settings**:
   - Enable **Store Tokens** if you need to pass the upstream token downstream.
   - Leave **Trust Email** unchecked unless your IdP guarantees email uniqueness.

7. Click **Save**.

### Step 3 — Configure a First Login Flow (optional but recommended)

The *First Login Flow* controls what happens when a user authenticates via the IdP for the very first time in Keycloak.

1. Still in **Identity Providers**, click the alias you just created → **Settings** tab.
2. Under **First Login Flow**, select `first broker login` (the default).
3. To automatically link the upstream account to an existing Keycloak user (matched by email), navigate to **Authentication → Flows → first broker login** and ensure the **Review Profile** and **Automatically Set Existing User** steps are enabled as needed.

> **Tip:** It is advisable to require profile review on first login so that display names and emails are correctly populated.

### Step 4 — Map upstream attributes to Keycloak attributes

Keycloak needs to know how to translate claims from the upstream token into its own user model.

1. Open **Identity Providers → `<your-alias>` → Mappers** tab.
2. Click **Add mapper** and create mappers for the attributes your organization uses. Common examples:

   | Mapper Type | Purpose |
   |---|---|
   | `Username Template Importer` | Map the upstream `sub` or `preferred_username` claim to the Keycloak username |
   | `Attribute Importer` | Copy `email`, `given_name`, `family_name` into the Keycloak user profile |
   | `Hardcoded Role` | Automatically assign a Keycloak role to *all* users from this IdP (see [RBAC](./rbac.md)) |
   | `Role Template Mapper` | Map an upstream claim (e.g. a group membership) to a specific Keycloak role |

> Role mapping is covered in detail in the [Role-based Access Control](./rbac.md) chapter.

---

## Connecting GitHub Enterprise

GitHub Enterprise (both Server and Cloud) uses OAuth 2.0. Keycloak ships with a built-in **GitHub** provider type that handles the OAuth 2.0 flow and knows how to fetch user profile information from the GitHub API. For GitHub Enterprise Server you additionally supply the base URL of your self-hosted instance.

### Step 1 — Register an OAuth App in GitHub Enterprise

1. In your GitHub Enterprise organization, navigate to  
   **Settings → Developer settings → OAuth Apps → New OAuth App**  
   (or at the organization level: **Organization Settings → Developer settings → OAuth Apps**).
2. Fill in the form:

   | Field | Value |
   |---|---|
   | **Application name** | e.g. `openDuT` |
   | **Homepage URL** | `https://<your-opendut-domain>/` |
   | **Authorization callback URL** | `https://auth.<your-domain>/realms/opendut/broker/github-enterprise/endpoint` |

   Replace `github-enterprise` with whatever alias you choose in Step 2.

3. Click **Register application**.
4. On the next screen, note the **Client ID** and generate a **Client Secret** — you will need both in Step 2.

### Step 2 — Add the GitHub Provider in Keycloak

1. Log in to Keycloak and switch to the **`opendut`** realm.
2. Navigate to **Identity Providers** and click **Add provider**.
3. Select **GitHub** from the list of built-in social providers.
4. Fill in the form:

   | Field | Value |
   |---|---|
   | **Alias** | e.g. `github-enterprise` |
   | **Display name** | e.g. `GitHub Enterprise` |
   | **Client ID** | The Client ID from Step 1 |
   | **Client Secret** | The Client Secret from Step 1 |

5. For **GitHub Enterprise Server** (self-hosted instance), scroll down to **Advanced Settings** and set both URL fields to your GitHub Enterprise Server hostname, e.g.:

   | Field | Value |
   |---|---|
   | **Base Url** | `https://github.example.com` |
   | **API URL** | `https://github.example.com/api/v3` |

   Keycloak uses **Base Url** to construct the authorization and token endpoints:
   - Authorization: `<base-url>/login/oauth/authorize`
   - Token: `<base-url>/login/oauth/access_token`

   **API URL** is used separately to fetch the authenticated user's profile (e.g. `<api-url>/user`). On GitHub Enterprise Server this is always `<your-hostname>/api/v3`.

   Leave both fields empty if you are using **GitHub Enterprise Cloud** (github.com).

6. Click **Save**.

### Step 3 — Map GitHub Attributes to Keycloak

The built-in GitHub provider imports basic profile fields automatically. If you need additional mappings (e.g. mapping a GitHub organization or team membership to the `opendut-user` role):

1. Open **Identity Providers → `github-enterprise` → Mappers**.
2. To grant all GitHub Enterprise users the `opendut-user` role, add a **Hardcoded Role** mapper as described in [Step 4 / Option A of the RBAC chapter](./rbac.md#option-a--hardcode-the-role-for-all-users-from-an-idp).
3. To restrict access to a specific GitHub organization or team, use the Post Login Flow approach from [Step 5 of the RBAC chapter](./rbac.md#step-5--restrict-the-idp-to-deny-login-for-users-without-the-role) combined with the role assignment above.

> **Note:** GitHub's OAuth 2.0 token does not include group or team membership claims by default. If you need team-based access control, the recommended approach is to assign the `opendut-user` role via a Hardcoded Role mapper (granting access to all authenticated GitHub Enterprise users) and rely on GitHub Enterprise itself to control who can authenticate against your OAuth App (e.g. by enforcing organization membership or SSO policies there).

### Step 4 — First Login Flow and Post Login Flow

Leave **First Login Flow** set to `first broker login` (the default).

If you want to restrict access by role, attach the post login flow created in the [RBAC chapter](./rbac.md#step-5--restrict-the-idp-to-deny-login-for-users-without-the-role):

1. Open **Identity Providers → `github-enterprise` → Settings**.
2. Set **Post Login Flow** to `opendut-post-login`.
3. Save.

---

## Connecting an LDAP / Active Directory Directory Service

For LDAP-backed user stores, Keycloak uses **User Federation** rather than Identity Providers.

1. Navigate to **User Federation** in the left-hand menu.
2. Click **Add provider → ldap**.
3. Fill in the connection details:

   | Field | Example value |
   |---|---|
   | **Vendor** | `Active Directory` (or `Other` for generic LDAP) |
   | **Connection URL** | `ldap://dc.example.com:389` |
   | **Bind DN** | `cn=keycloak-bind,ou=ServiceAccounts,dc=example,dc=com` |
   | **Bind Credentials** | The bind account password |
   | **Users DN** | `ou=Users,dc=example,dc=com` |
   | **Username LDAP attribute** | `sAMAccountName` (AD) or `uid` (OpenLDAP) |

4. Click **Test connection** and **Test authentication** to verify the settings.
5. Click **Save**, then **Synchronize all users** to perform the initial import.

### Map LDAP Groups to Keycloak Roles

After federation is working, map your LDAP/AD security groups to Keycloak roles so that group membership automatically grants the required openDuT access:

1. Open **User Federation → `<your-ldap-provider>` → Mappers**.
2. Click **Add mapper** and select **role-ldap-mapper**.
3. Configure:
   - **LDAP Roles DN:** the DN of the group/role container in LDAP.
   - **Membership LDAP Attribute:** `member` (AD) or `memberUid` (POSIX).
   - **Client ID:** leave blank for realm roles, or select the specific openDuT client.
4. Save and synchronize roles.

> For the full list of supported LDAP mapper types see the [Keycloak documentation on LDAP mappers](https://www.keycloak.org/docs/latest/server_admin/#_ldap_mappers).

---

## Verifying the Configuration

1. Open a private/incognito browser window and navigate to the openDuT LEA UI at `https://<your-domain>/`.
2. You should see a **Login with `<Display name>`** button on the Keycloak login page.
3. Click it and authenticate with a user from your upstream IdP.
4. After successful login you are redirected back to LEA.
5. In the Keycloak admin console, confirm that a user entry was created under **Users** in the `opendut` realm and that the expected roles are present.

If the login button does not appear, verify that the identity provider is marked as **Enabled** in its settings.

---

## Next Steps

Once users can authenticate via the upstream IdP, proceed to [Role-based Access Control](./rbac.md) to ensure that only users with the appropriate role can access openDuT components.

