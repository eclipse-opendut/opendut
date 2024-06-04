# Restbus Simulation

### Summary
This module provides a restbus-simulation to the OpenDuT user and it is split into an **_ARXML parser (AXP)_**
sub-module and a **_restbus-simulation (RSIM)_** sub-module. The AXP parses an ARXML file and the parsed data
can be used by the RSIM to establish a working simulation that knows about all the Frames/PDUs/Signals and 
handles the simulation of them.

Live changes of signals/timings/... shall be implemented by the end-user, which can use a simple API of the RSIM to 
define changes in an abstract way. The goal is not to achieve the same functionality as the
well-known restbus-simulation tool. Instead, the OpenDuT user should have an easy possibility 
to simulate a base environment that improves testing.

Current implementation state:
  - AXP -> Done
  - RSIM -> Base implementation done and tested successfully. Have to extend it
    to handle all types of PDUs and do some more modifications.
  - RSIM API -> Todo

### ARXML Parser (AXP) sub-module
This module parses an ARXML (Autosar XML) file and extracts all values necessary for a restbus-simulation. 
First, the [autosar-data crate](https://crates.io/crates/autosar-data/0.9.0) is used for parsing an ARXML file.
Then, important data is extracted from the parsed data and some post-processing is made. The resulting 
data is stored in structures, which basically represent different 
[Autosar Elements](https://www.autosar.org/fileadmin/standards/R22-11/CP/AUTOSAR_TPS_SystemTemplate.pdf). 

Parsing and post-processing a big ARXML file can take a long time. For example, for a ~300 MB ARXML file, we need
around 40 seconds on a standard laptop. Therefore, the parser can be instructed to serialize the resulting structures and store them into 
a file. This enables a
very quick re-establishment of everything, since we do not need to parse and post-process data for a second time.
Instead, the next time we run the program, we just can deserialize the data, which takes less than one second.

The resulting structures can be modified before passing them to the RSIM. There is no direct API for creation/modification of
structures implemented yet, but manually modifying the structures by making use of AXP helper methods is easily possible.
If a use-case for structure modification exists, then a later API might be implemented, which
should not take a lot of time. However, currently, the idea is that everything is already properly defined through the 
ARXML file.

### Restbus-Simulation (RSIM) sub-module
The RSIM can be fed with the structures coming from the AXP. With these structures, the RSIM exactly knows how 
Frames/PDUs/Signals/Timings/Initial Values, ... look like. It handles all the lower-level things and controls what is and how it is send
to the Bus. The user always has just an abstract overview of everything. See the **Configuration** section for learning about the configuration of everything.

The RSIM makes use of the [Linux SocketCAN Broadcast Manager (BCM)](https://www.kernel.org/doc/Documentation/networking/can.txt),
which handles all the timing of (optionally) periodically sent messages. The BCM is setup and modified 
via **BCM sockets**, in which we can define message bytes and their timing information. The Kernel handles then 
the correct message transmission + timing. Furthermore, the BCM will also be used 
to dynamically modify messages and their timing during runtime.

The user itself has to provide the status changing logic.
With a simple API (see next section), the user can instruct the RSIM to modify the data that is sent to the bus. 
The user has then control over single signals, timings, and more, by using the API. **For example**, we have a periodically sent message definining
that the car's doors are locked. The RSIM completely handles the periodic sending with the right timing etc. 
The user can then tell the RSIM that the status
has changed by instructing the RSIM to change the signal (lock status) of that particular message or all messages 
referencing that signal. This will be possible 
with a simple API call like "Change _Signal CarLock_ to 0 (false)". As a result, the message/s referencing the signal
will be adapted automatically by the RSIM and the user does not need to know about any low-level implementation.
The details to the API will follow and will be 
defined in the **RSIM API** section.
Right now no API exists, and the RSIM just plays Frames with initial values to the Bus.

### RSIM API
Idea to discuss:
  - TODO. 

### Configuration
Currently, the implementation is somewhat independent of OpenDuT. In the future, everything will be integrated
including the configuration. You can see in [main.rs](https://github.com/eclipse-opendut/opendut/blob/restbus-simulation/opendut-edgar/restbus-simulation/src/main.rs)
how to establish a restbus-simulation without configuration.

### References:
  - [autosar-data crate](https://crates.io/crates/autosar-data/0.9.0)
  - [Autosar System Template](https://www.autosar.org/fileadmin/standards/R22-11/CP/AUTOSAR_TPS_SystemTemplate.pdf)
  - [Linux SocketCAN Broadcast Manager (BCM)](https://www.kernel.org/doc/Documentation/networking/can.txt)

