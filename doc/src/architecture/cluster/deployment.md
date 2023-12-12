
# Cluster

## Cluster Creation

```mermaid
sequenceDiagram
    participant LEA
    participant carl as CARL
    LEA->>carl: StoreClusterConfiguration
    activate carl
    carl->>carl: PersistClusterConfiguration
    carl-->>LEA: StoreClusterConfigurationSuccess
    deactivate carl
```

```
{{#include ../../../../opendut-types/proto/opendut/types/cluster/cluster.proto:ClusterConfiguration}}
```

## Cluster Deployment

```
{{#include ../../../../opendut-types/proto/opendut/types/cluster/cluster.proto:ClusterAssignment}}
```
```
{{#include ../../../../opendut-types/proto/opendut/types/cluster/cluster.proto:PeerClusterAssignment}}
```
