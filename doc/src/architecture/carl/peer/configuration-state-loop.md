# Configuration/State Loop

When the user configures a Peer or a Cluster, LEA/CLEO sends us a `PeerDescriptor` or `ClusterDescriptor` respectively.

To instruct EDGAR what configuration it should roll out, we send it a `PeerConfiguration`, which is derived from these `Descriptors` and additional information that CARL may have.

EDGAR reports back a `EdgePeerConfigurationState` to indicate how far along it is in rolling out this `PeerConfiguration`, as well as its general health.

This `EdgePeerConfigurationState` is then compared to the `PeerConfiguration` to determine what configuration tasks are still in progress.
Along with the peer's general health information, this is then reported to LEA/CLEO in a PeerStatusReport.

![Illustration how Descriptors, Configuration, State and StatusReport are sent between programs.](img/peer-state-configuration.svg)


## PeerConfigurationState

With the `PeerConfigurationState` we want to tell if the edge device has applied all required configuration tasks and is considered `ready`.
To determine the `PeerConfigurationState` we need to compare the `PeerConfiguration` and the `EdgePeerConfigurationState` (feedback of the edge device).
When a Parameter is "Present" in PeerConfiguration and "Absent" in EdgePeerConfigurationState, it is assumed to be "Creating", unless an error has been reported. This works similarly for "Removing", "Present" and "Absent".

| PeerConfiguration | EdgePeerConfigurationState | PeerConfigurationState | Review |
|-------------------|----------------------------|------------------------|--------|
| present           | present + OK               | PRESENT                | no     |
| absent            | absent  + OK               | ABSENT                 | no     |
| present           | absent  + OK               | CREATING               | no     |
| absent            | present + OK               | REMOVING               | no     |
| present           | absent  + error            | CREATING + Error       | yes    |
| absent            | present + error            | REMOVING + Error       | yes    |

The last column indicates if this is a transitional state that can be expected to resolve itself (no), or if user intervention is likely required (yes).

### Detected state unknown in EdgePeerConfigurationState

The state might be unknown if e.g. a dependency for creating/removing the configuration parameter is not yet fulfilled.
It is also unknown if an error occurred while checking the state.

| PeerConfiguration | EdgePeerConfigurationState | PeerConfigurationState               | Review |
|-------------------|----------------------------|--------------------------------------|--------|
| present           | unknown  + error           | CHECKFAILED + Error                  | yes    |
| absent            | unknown  + error           | CHECKFAILED + Error                  | yes    |
| present           | unknown  + error           | INCARNATING + WaitingForDependencies | no     |
| absent            | unknown  + error           | INCARNATING + WaitingForDependencies | no     |


### Configuration parameter is missing at one side

| PeerConfiguration | EdgePeerConfigurationState | PeerConfigurationState | Comment                                                   | Review |
|-------------------|----------------------------|------------------------|-----------------------------------------------------------|--------|
| present           | not appear                 | INCARNATING            | backend expects it, edge hasn't reported it yet           | no     |
| absent            | not appear                 | INCARNATING            | backend expects absent, edge hasn't checked/reported      | no     |
| not appear        | absent  + OK               | ABSENT                 | both sides agree it's absent                              | no     |
| not appear        | present + OK               | PRESENT                | backend doesn't expect, edge reports present              | maybe  |
| not appear        | present + error            | REMOVING + Error       | error while removing obsolete parameter                   | yes    |
| not appear        | absent  + error            | CHECKFAILED + Error    | error while checking/removing absent (obsolete) parameter | yes    |
| not appear        | unknown  + error           | CHECKFAILED + Error    | check failed on an obsolete parameter                     | yes    |

Special states:
* The INCARNATING state indicates that the backend expects a parameter, but the edge device has not yet checked or reported it. 
  This is a transitional state, might have an error set and is not considered 'ready'.
* The CHECKFAILED state indicates that an error occurred while checking the presence/absence of a configuration parameter.
  This is considered a blocking error and requires user intervention.



## Ideas behind this design

* Treat EDGAR as a dumb runner – it should not make own decisions.
  Rather it is given a complete set of configuration parameters (`PeerConfiguration`) to apply.
  CARL makes the decisions.
  * It needs to be possible to remove configuration parameter deployments generically,
    by marking them `Absent` rather than `Present`.
  * There will be Tasks, akin to EDGAR Setup, which can check whether a configuration parameter deployment is present/absent,
    and know how to apply or unapply this configuration.

* EDGAR can report how far it has applied the PeerConfiguration as a `EdgePeerConfigurationState`.
  The EdgePeerConfigurationState needs to be linkable to the PeerConfiguration.

* UIs should get a report on the state of a peer (`PeerStatusReport`).
  This PeerStatusReport needs to be derived from the EdgePeerConfigurationState.

* It needs to be possible to designate dependencies between configuration parameters,
  as e.g. an interface cannot be joined to a bridge, if the bridge does not yet exist.

* EDGAR should be able to restart or reconnect, and then apply the newest `PeerConfiguration` without hiccups.
  As such, the `PeerConfiguration` may not depend on a supposed state of EDGAR.

* When a `PeerConfiguration` is applied, this needs to be idempotent.
