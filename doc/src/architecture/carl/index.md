# CARL

```plantuml
@startuml
node "LEA/CLEO" as UI

frame CARL {
  agent ClusterManager
  agent PeerManager
  portout VpnManagementClient
  agent PeerMessagingBroker
  agent ResourcesManager
}
node "VPN\nManagement\nServer" as VpnManagementServer
node EDGAR
database Persistence

UI --> ClusterManager
UI --> PeerManager

ClusterManager --> PeerMessagingBroker
ClusterManager ---> ResourcesManager
PeerManager -- VpnManagementClient
PeerManager ---> ResourcesManager
ResourcesManager <- PeerMessagingBroker
ResourcesManager --> Persistence

PeerMessagingBroker <==> EDGAR : Stream (PeerConfiguration)

VpnManagementClient -> VpnManagementServer

@enduml
```
