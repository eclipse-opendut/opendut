## System Context View

> **Note:** At the moment, the authorization concept is a draft because users and administrators cannot be distinguished, yet. The diagrams below illustrate the intended architecture.


```plantuml
@startuml
!include <C4/C4_Context>

' Actors
Person(admin, "Administrator", "Manages the system")
Person(user, "User", "Uses OpenDuT services")
System_Ext(idp, "Upstream Identity Provider", "External OIDC provider")
Person(partner_admin, "Partner Administrator", "Operates partner edge device")

' System
System_Boundary(opendut_boundary, "OpenDuT System") {
  System(opendut, "OpenDuT", "Distributed testbed for automotive networks")
  System(netbird, "NetBird", "VPN Service") <<third-party>>
  System(keycloak, "Keycloak", "Identity Provider") <<third-party>>
  System(edgar, "EDGAR", "Peer Node")
  System(dut, "DuT", "Device under Test") <<third-party>>
}

' External partner systems
System_Ext(partner_edgar, "Partner EDGAR", "Partner-operated edge device running EDGAR")
System_Ext(partner_dut, "Partner DuT", "Partner Device under Test") <<third-party>>

' Relationships
Rel(admin, opendut, "Manages via LEA UI/API", "TLS/GRPC")
Rel(user, opendut, "Uses via LEA UI/API", "TLS/GRPC")
Rel(opendut, idp, "Delegates authentication", "OIDC/SAML/TLS")
Rel(opendut, netbird, "VPN management", "REST API/Bearer Token")
Rel(opendut, keycloak, "Authentication/Authorization", "OIDC/OAuth2")
Rel(opendut, edgar, "Manages peer nodes", "GRPC/TLS")
Rel(edgar, dut, "Manages network device", "CAN, ETH")
Rel(partner_admin, partner_edgar, "Operates partner edge device")
Rel(partner_edgar, partner_dut, "Manages partner network device", "CAN, ETH")
Rel(partner_edgar, opendut, "Connects to backend", "GRPC, OAuth2")
Rel(edgar, partner_edgar, "Peer communication", "WireGuard")
Rel(dut, partner_dut, "Forward DuT traffic", "CAN, ETH")
Rel(partner_dut, dut, "Forward DuT traffic", "CAN, ETH")

@enduml
```