# VIPER

VIPER provides a test execution platform.


This is how we plan to integrate VIPER into the openDuT communication:

```plantuml
participant "LEA/CLEO" as UI
participant CARL
participant "VIPER-Runtime" as VIPER_RT
participant EDGAR

== Defining a test suite source ==

UI --> CARL: Store source definition (SourceDescriptor)
UI <-- CARL: Success

== Parametrizing a test suite run ==

UI --> CARL: Request suite names & parameters
CARL -> VIPER_RT: Source definitions
CARL <- VIPER_RT: Suite names & parameters
UI <-- CARL: Suite names & parameters

UI --> CARL: Store selected suite name & \n parameter values (RunDescriptor)
UI <-- CARL: Success

== Running a test suite ==

UI --> CARL: Trigger suite run (RunDeployment)
CARL --> EDGAR: Selected suite name & \n source & parameter values
EDGAR -> VIPER_RT: Selected suite name & \n source & parameter values
EDGAR <- VIPER_RT: Test results
CARL <-- EDGAR: Test results
UI <-- CARL: Test results
```

Network calls are indicated by dotted arrows.  
Function calls are indicated by solid arrows. (VIPER-Runtime is a library, included by both CARL and EDGAR.)
