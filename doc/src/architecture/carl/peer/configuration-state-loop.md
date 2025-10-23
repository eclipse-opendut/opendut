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

| PeerConfiguration | Detected | EdgePeerConfigurationState | PeerConfigurationState | Review |
|-------------------|----------|----------------------------|------------------------|--------|
| present           | present  | present                    | PRESENT                | no     |
| absent            | absent   | absent                     | ABSENT                 | no     |
| present           | absent   | absent                     | CREATING               | no     |
| absent            | present  | present                    | REMOVING               | no     |
| present           | absent   | CreatingFailed             | CreatingFailed         | yes    |
| absent            | present  | RemovingFailed             | RemovingFailed         | yes    |

The last column indicates if this is a transitional state that can be expected to resolve itself (no), or if user intervention is likely required (yes).

### Detected state unknown in EdgePeerConfigurationState

The state might be unknown if e.g. a dependency for creating/removing the configuration parameter is not yet fulfilled.
It is also unknown if an error occurred while checking/asserting the state.

| PeerConfiguration | Detected | EdgePeerConfigurationState | PeerConfigurationState | Review |
|-------------------|----------|----------------------------|------------------------|--------|
| present           | unknown  | CheckPresentFailed         | CheckPresentFailed     | yes    |
| absent            | unknown  | CheckAbsentFailed          | CheckAbsentFailed      | yes    |
| present           | unknown  | WaitingForDependencies     | WaitingForDependencies | no     |
| absent            | unknown  | WaitingForDependencies     | WaitingForDependencies | no     |


### Configuration parameter is missing at one side

| PeerConfiguration | Detected | EdgePeerConfigurationState           | PeerConfigurationState | Comment                                              | Review |
|-------------------|----------|--------------------------------------|------------------------|------------------------------------------------------|--------|
| present           | nothing  | not appear                           | CREATING               | backend expects it, edge hasn't reported it yet      | no     |
| absent            | nothing  | not appear                           | REMOVING               | backend expects absent, edge hasn't checked/reported | no     |
| not appear        | absent   | absent                               | <drop>                 | both sides agree it's absent                         | no     |
| not appear        | present  | present                              | <drop>                 | backend doesn't expect, edge reports present         | maybe  |
| not appear        | present  | RemovingFailed                       | <drop>                 | error while removing obsolete parameter              | yes    |
| not appear        | absent   | CreatingFailed                       | <drop>                 | error while creating (obsolete) parameter            | yes    |
| not appear        | unknown  | CheckPresentFailed/CheckAbsentFailed | <drop>                 | check failed on an obsolete parameter                | yes    |

### Configuration Loop per Parameter

1. Check: 
   detected_state: Absent, present, unknown (is represented by waiting for dependencies error)
   detected_error: CheckAbsentFailed, CheckPresentFailed, WaitingForDependencies
2. Execution: make_present/make_absent
   execution_result: present/absent, task was created/removed
   execution_error: CreatingFailed/RemovingFailed 
3. Assert: 
   detected_state: Absent, present
   detected_error: CheckAbsentFailed, CheckPresentFailed -> detected state unknown

Special states:
* Final states: Present, Absent (configured as desired)
* Transitional states: Creating, Removing, WaitingForDependencies
* Error states: CreatingFailed, RemovingFailed, CheckPresentFailed, CheckAbsentFailed
* The <drop> state indicates that this should be ignored, logged and not included in the `PeerConfigurationState`.

If all parameters are in final states, the overall PeerConfigurationState is `ready`.


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
