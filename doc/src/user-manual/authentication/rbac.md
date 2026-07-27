# Role-based Access Control (RBAC)

> **Current status:** CARL does not yet enforce role checks on incoming requests.
> Role-based access restriction is on the roadmap and will be implemented in a future release.
> Until then, access control must be enforced **at the Keycloak level** using the mechanisms described in [Step 5](#step-5--restrict-the-idp-to-deny-login-for-users-without-the-role) below.

openDuT uses Keycloak roles to define which users are permitted to access openDuT components — LEA, CLEO, and EDGAR.
Because role enforcement currently takes place exclusively inside Keycloak, it is important to configure Keycloak correctly so that only authorized users can obtain a valid session in the first place.

This chapter describes how to:

1. Understand the intended openDuT role model.
2. Create the role in Keycloak (if it does not yet exist).
3. Assign the role to users manually.
4. Automatically assign the role based on upstream IdP group membership.
5. Restrict an upstream identity provider so that only matching users can log in at all (the only currently effective enforcement point).

---

## The openDuT Role

The openDuT realm in Keycloak uses the realm role **`opendut-user`** to identify users who are permitted to use openDuT.

> **Note:** Because CARL does not yet inspect this role at runtime, the role assignment itself has no technical effect on access today. Its purpose is to prepare the configuration for the upcoming role-enforcement feature and to drive the Keycloak-level access restriction described in Step 5. Operators are strongly encouraged to configure this now so that the transition to enforced RBAC requires no additional changes.

> The exact role name may change once CARL-side enforcement is implemented. Check the release notes or the `auth` section of `carl.toml` for updates.

---

## Step 1 — Create the Role in Keycloak

If the role does not already exist (the openDuT provisioning step usually creates it), create it manually:

1. Log in to Keycloak and switch to the **`opendut`** realm.
2. Navigate to **Realm roles** in the left-hand menu.
3. Click **Create role**.
4. Set **Role Name** to `opendut-user` and optionally add a description.
5. Click **Save**.

---

## Step 2 — Assign the Role to Individual Users

For small teams or initial testing you can assign the role directly to individual users:

1. Navigate to **Users** and open the user you want to grant access to.
2. Open the **Role mappings** tab.
3. Under **Assign role**, search for `opendut-user`, select it, and click **Assign**.

The user can now log in to openDuT components.

---

## Step 3 — Assign the Role via a Keycloak Group (recommended for teams)

Managing roles per-user does not scale well. Using Keycloak groups keeps role assignment maintainable:

1. Navigate to **Groups** and click **Create group**.
2. Name the group, e.g. `opendut-users`.
3. Open the new group and go to the **Role mappings** tab.
4. Assign the `opendut-user` role to the group.
5. Add users to this group (via **Users → `<user>` → Groups** or **Groups → `<group>` → Members**).

Every member of the group automatically inherits the `opendut-user` role.

---

## Step 4 — Automatically Assign the Role from an Upstream Identity Provider

If you have [configured an upstream IdP](./identity-provider.md), you can map upstream group or role claims to the `opendut-user` role automatically during login. This means no manual role assignment is needed — access is fully driven by your organization's existing directory.

### Option A — Hardcode the role for all users from an IdP

Use this approach when *every* user authenticated via a specific IdP should have access:

1. Navigate to **Identity Providers → `<your-alias>` → Mappers**.
2. Click **Add mapper**.
3. Set:
   - **Name:** `assign-opendut-user`
   - **Mapper Type:** `Hardcoded Role`
   - **Role:** `opendut-user`
4. Save.

From now on, every user who logs in through this IdP will automatically receive the `opendut-user` role.

### Option B — Map an upstream group/role claim to the Keycloak role

Use this approach when only *specific groups* from your upstream IdP should have access:

1. Ensure your upstream IdP includes group membership in its token. For example, Azure AD / Entra ID can emit a `groups` claim containing group object IDs.
2. Navigate to **Identity Providers → `<your-alias>` → Mappers**.
3. Click **Add mapper**.
4. Set:
   - **Name:** `map-upstream-group-to-opendut-user`
   - **Mapper Type:** `Claim to Role`  
     *(If this type is not available, use `Role Template Mapper` or `Oidc Role Name Mapper` depending on your Keycloak version.)*
   - **Claim:** the name of the upstream claim that carries group/role information, e.g. `groups`
   - **Claim Value:** the specific value that indicates membership, e.g. the group's object ID or name
   - **Role:** `opendut-user`
5. Save.

Only users whose token contains the matching claim value will receive the `opendut-user` role.

> **LDAP / Active Directory:** If you use LDAP User Federation, configure a **role-ldap-mapper** as described in [Configuring an Identity Provider](./identity-provider.md#map-ldap-groups-to-keycloak-roles) to map your AD security group to the `opendut-user` realm role.

---

## Step 5 — Restrict the IdP to Deny Login for Users Without the Role

The steps above set up the role model that will be used for enforcement once CARL-side role checking is available. However, at this time **Keycloak is the only place where access can actually be blocked**. Without the configuration below, users who lack the `opendut-user` role can still authenticate via Keycloak and reach openDuT components freely.

Use one of the following approaches to enforce the restriction at the Keycloak level:

### Approach A — Post Login Flow with Role Condition (recommended for Keycloak ≥ 26)

In Keycloak 26, the cleanest way to block unauthorized users at login is to attach a **Post Login Flow** to the identity provider.
The **First Login Flow** should remain set to the built-in `first broker login` (or a copy of it) — changing that flow is not necessary and can break account linking.

#### Step 1 — Create the Post Login Flow

1. Navigate to **Authentication → Flows** and click **Create flow**.
2. Set:
   - **Name:** e.g. `opendut-post-login`
   - **Type:** `Basic`
3. Inside the new flow, click **Add sub-flow**:
   - **Name:** e.g. `check-access`
   - **Type:** `Conditional`
4. Inside the `check-access` sub-flow, click **Add step** and select **Condition - User Role**:
   - **User role:** `opendut-user`
   - **Negate output:** `OFF`
   - Set requirement to **REQUIRED**.
5. Still inside the `check-access` sub-flow, click **Add step** again and select **Allow Access**:
   - Set requirement to **REQUIRED**.

The resulting flow structure should look like this:

```
opendut-post-login
  └── check-access  (Conditional sub-flow)   CONDITIONAL
        ├── Condition - User Role             REQUIRED
        └── Allow Access                      REQUIRED
```

**How it works:** After the upstream IdP login, Keycloak evaluates the condition. When the user *has* the `opendut-user` role the sub-flow executes and hits **Allow Access**, granting the session. When the user *lacks* the role the condition returns `false`, the sub-flow is skipped entirely, and Keycloak finds no other path to grant access — the login is blocked.

#### Step 2 — Attach the Flow to the Identity Provider

1. Navigate to **Identity Providers → `<your-alias>` → Settings**.
2. Leave **First Login Flow** set to `first broker login` (the default).
3. Set **Post Login Flow** to `opendut-post-login`.
4. Save.

Users who authenticate via the IdP but do not hold the `opendut-user` role will now be rejected by Keycloak and never reach openDuT components.


---

## Verifying Access Control

Because CARL does not yet enforce roles itself, verification focuses on confirming that the **Keycloak-level restriction** (Step 5) is working correctly.

### Positive test (user with role)
1. Add the `opendut-user` role to a test user (or ensure the user is in the correct upstream group).
2. Log in to LEA — you should be granted a Keycloak session and reach the application.
3. Run a CLEO command (e.g. `opendut-cleo list peers`) — it should succeed.

### Negative test (user without role)
1. Create or use a test user that does **not** have the `opendut-user` role.
2. If you have configured the authentication flow from Step 5, the user should be rejected by Keycloak during login and never reach openDuT components.
3. Without the Step 5 flow, the user will be able to obtain a Keycloak session — this is the gap that the upcoming CARL-side role enforcement will close.
4. Verify in the Keycloak admin console under **Events** (enable **Login events** in **Realm Settings → Events**) that the login attempt is logged.

---

## Summary

| Goal | Mechanism | Enforced by |
|---|---|---|
| Grant access to all users of an IdP | Hardcoded Role mapper on the Identity Provider | Keycloak (today) / CARL (future) |
| Grant access to a subset of IdP users | Claim-to-Role mapper matching a specific group claim | Keycloak (today) / CARL (future) |
| Grant access based on AD group membership | LDAP role mapper pointing to the AD security group | Keycloak (today) / CARL (future) |
| Block non-authorized users at Keycloak login | Custom Authentication Flow with Condition - User Role | **Keycloak only (current recommendation)** |

Until CARL-side role enforcement is available, configuring the Keycloak authentication flow (Step 5) is the **only reliable way** to prevent unauthorized users from accessing openDuT. All other steps prepare the role model for the future and are still recommended to configure now.




