# CARL

## Overview

```plantuml
@startuml
node "LEA/CLEO" as UI

frame CARL {
  agent ClusterManager
  agent PeerManager
  portout VpnManagementClient
  agent PeerMessagingBroker
  agent ResourceManager
}
node "VPN\nManagement\nServer" as VpnManagementServer
node EDGAR
database Persistence

UI --> ClusterManager
UI --> PeerManager

ClusterManager --> PeerMessagingBroker
ClusterManager ---> ResourceManager
PeerManager -- VpnManagementClient
PeerManager ---> ResourceManager
ResourceManager <- PeerMessagingBroker
ResourceManager --> Persistence

PeerMessagingBroker <==> EDGAR : Stream (PeerConfiguration)

VpnManagementClient -> VpnManagementServer

@enduml
```

## ResourceManager

![](img/resources-manager.svg)
