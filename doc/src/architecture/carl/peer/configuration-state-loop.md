# Configuration/State Loop

When the user configures a Peer or a Cluster, LEA/CLEO sends us a `PeerDescriptor` or `ClusterDescriptor` respectively.

To instruct EDGAR what configuration it should roll out, we send it a `PeerConfiguration`, which is derived from these `Descriptors` and additional information that CARL may have.

EDGAR reports back a `PeerConfigurationState` to indicate how far along it is in rolling out this `PeerConfiguration`, as well as its general health.

This `PeerConfigurationState` is then compared to the `PeerConfiguration` to determine what configuration tasks are still in progress.
Along with the peer's general health information, this is then reported to LEA/CLEO in a PeerStatusReport.

![Illustration how Descriptors, Configuration, State and StatusReport are sent between programs.](img/peer-state-configuration.svg)


## Ideas behind this design

* Treat EDGAR as a dumb runner – it should not make own decisions.
  Rather it is given a complete set of configuration parameters (`PeerConfiguration`) to apply.
  CARL makes the decisions.
  * It needs to be possible to remove configuration parameter deployments generically,
    by marking them `Absent` rather than `Present`.
  * There will be Tasks, akin to EDGAR Setup, which can check whether a configuration parameter deployment is present/absent,
    and know how to apply or unapply this configuration.

* EDGAR can report how far it has applied the PeerConfiguration as a `PeerConfigurationState`.
  The PeerConfigurationState needs to be linkable to the PeerConfiguration.

* UIs should get a report on the state of a peer (`PeerStatusReport`).
  This PeerStatusReport needs to be derived from the PeerConfigurationState.

* It needs to be possible to designate dependencies between configuration parameters,
  as e.g. an interface cannot be joined to a bridge, if the bridge does not yet exist.

* EDGAR should be able to restart or reconnect, and then apply the newest `PeerConfiguration` without hiccups.
  As such, the `PeerConfiguration` may not depend on a supposed state of EDGAR.

* When a `PeerConfiguration` is applied, this needs to be idempotent.
