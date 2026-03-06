# VIPER

VIPER provides a test execution platform.


This is how we plan to integrate VIPER into the openDuT communication:

```plantuml
participant "LEA/CLEO" as UI
participant CARL
participant EDGAR
participant "VIPER-Runtime" as VIPER
participant Source

== Defining a test suite source ==

UI --> CARL: Store source definition\n(SourceDescriptor)
UI <-- CARL: Success


== Parametrizing a test suite run ==

UI --> CARL: GetViperTestSuiteDescriptor(source_id)
CARL -> VIPER: SourceDescriptor

activate VIPER
VIPER --> Source: Fetch
VIPER <-- Source: Source code
VIPER -> VIPER: Compile
CARL <- VIPER: ParameterDescriptors
deactivate VIPER

UI <-- CARL: ViperTestSuiteDescriptor

note over UI: User enters parameter values

UI --> CARL: StoreViperTestRunDescriptor
UI <-- CARL: Success


== Running a test suite ==
UI --> CARL: StoreViperTestRunDeployment (triggers test run)
CARL --> UI: Success

CARL --> EDGAR: Selected suite name & \n source & parameter values

EDGAR -> VIPER: Selected suite name & \n source & parameter values

activate VIPER
VIPER --> Source: Fetch
VIPER <-- Source: Source code
VIPER -> VIPER: Compile & Run
EDGAR <- VIPER: Test results
deactivate VIPER

CARL <-- EDGAR: Test results

UI --> CARL: Request test completion state
UI <-- CARL: Test completion state
```

Network calls are indicated by dotted arrows.  
Function calls are indicated by solid arrows. (VIPER-Runtime is a library, included by both CARL and EDGAR.)
