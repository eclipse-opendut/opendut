# Hardware Setup

This is an example setup of openDuT to give you a concrete setup that should work.
We will assume an openDuT backend running locally on a PC and local connectivity for simplicity. Of course, it is one of the goals of openDuT to allow separating the ECUs, so a production setup does not have to look this way.


## Requirements

* 2x Raspberry Pi for [running EDGAR](https://opendut.eclipse.dev/book/user-manual/edgar/setup.html)
  We typically tested with models 3B+ and 4, using Raspberry Pi OS Lite.
  Older models should also work (throughput of the network interfaces is the limiting factor), but the [hotspot-based setup](https://github.com/eclipse-opendut/raspberry-pi-wireless-bootstrap) then doesn't work for them.

* An x86_64 Linux PC for [running the backend](https://opendut.eclipse.dev/book/user-manual/carl/setup.html)

* A switch (or router)
* 3x Ethernet cables

* For Ethernet communication: 2x USB-to-Ethernet adapters and 2x Ethernet cables
* For CAN communication: 2x USB-to-CAN adapters and 2x CAN cables


## Setup

High-level, you would connect the components like this:

1. Set up the Raspberry Pis with Raspberry Pi OS Lite and EDGAR.
2. Set up the Raspberry Pis and the PC with EDGAR and CARL respectively.
3. Set up the switch and connect the devices to it via 3x Ethernet cables.
4. Configure openDuT, so the EDGARs are in a cluster.
5. Connect the Raspberry Pis to the ECUs via the USB adapters and remaining cables.

See the rest of the documentation (mostly the parts linked above) for how to actually do that.
