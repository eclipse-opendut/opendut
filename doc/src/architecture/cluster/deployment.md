
# Cluster

## Cluster creation

```mermaid
sequenceDiagram
    participant LEA
    participant carl as CARL
    LEA->>carl: StoreClusterConfigration
    activate carl
    carl->>carl: PersistClusterConfigration
    carl-->>LEA: StoreClusterConfigrationSuccess
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
