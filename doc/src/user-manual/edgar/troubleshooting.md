# Troubleshooting Guide

## Symptom Diagnosis

### EDGAR Setup fails
Something failed before the EDGAR Service even started.  
→ See [EDGAR Setup Troubleshooting](setup.md#troubleshooting).

### EDGAR does not show up as online
The web-UI or CLI lists an EDGAR not as healthy.  
→ See [Troubleshooting EDGAR offline](#troubleshooting-edgar).


### The connection does not work
You cannot establish a connection with the ECU(s).

#### Check the interfaces are configured correctly

Run `ip link` and check that all of these interfaces exist:

* `br-opendut`
  EDGAR did not receive or roll out the configuration.  
  → See [Troubleshooting EDGAR](#troubleshooting-edgar).

* `wt0`
  The WireGuard tunnel has not been set up correctly by NetBird.  
  → See [Troubleshooting VPN connection](#troubleshooting-vpn-connection).

* If you configured an Ethernet interface:
  * The Ethernet interface itself.
    (Should show up with `master br-opendut` in the `ip link` output.)  
    → See [Troubleshooting the ECUs](#troubleshooting-the-ecus).
  * `gre-...` (ending in random letters and numbers)
    (Should show up with `master br-opendut` in the `ip link` output.)  
    → See [Troubleshooting configuration rollout](#troubleshooting-edgar).

* If you configured a CAN interface:
  * The CAN interface itself.  
    → See [Troubleshooting the ECUs](#troubleshooting-the-ecus).
  * `br-vcan-opendut`  
    → See [Troubleshooting EDGAR](#troubleshooting-edgar).

* `gre0`, `gretap0` and `erspan0`
  The Generic Routing Encapsulation (GRE) has not been setup correctly.  
  → See [Troubleshooting EDGAR](#troubleshooting-edgar).  
  → If it is still not working, check if the `ip_gre` and `gre` kernel modules are available and can be loaded.

#### Ping throughout the connection
* Ping `wt0` as described in [Troubleshooting VPN connection](#troubleshooting-vpn-connection).
* Ping `br-opendut`:
  1. Assign an IP address to `br-opendut` on each device, for example:
     ```sh
     ip address add 192.168.123.101/24 dev br-opendut  #on one device
     ip address add 192.168.123.102/24 dev br-opendut  #on the other device
     ```
     The IP addresses have to be in the same subnet.
  2. Ping the assigned IP address of the other device:
     ```sh
     ping 192.168.123.102  #on one device
     ping 192.168.123.101  #on the other device
     ```
     If the ping works, the connection via openDuT should work.
     If it still does not, see [Troubleshooting the ECUs](#troubleshooting-the-ecus).


## Troubleshooting

### Troubleshooting EDGAR
* If the setup completed, but EDGAR does not show up as Healthy in LEA/CLEO, see:
  ```shell
  journalctl -u opendut-edgar
  ```

* Sometimes it helps to restart the EDGAR service:
  ```shell
  # Restart service
  sudo systemctl restart opendut-edgar

  # Check status
  systemctl status opendut-edgar
  ```

* Try rebooting the operating system.
  This clears out the interfaces, forcing EDGAR and NetBird to recreate them.

* When the configuration does not get rolled out, it can help to redeploy the cluster.
  In the web-UI or CLI, undeploy the cluster and then re-deploy it shortly after.


### Troubleshooting VPN connection

* See `/opt/opendut/edgar/netbird/netbird status --detail`.
  The remote peers should be listed as "Connected".  

* Check the NetBird logs for errors:
  ```shell
  cat /var/lib/netbird/client.log
  cat /var/lib/netbird/netbird.err
  cat /var/lib/netbird/netbird.out
  ```
  
* Try pinging `wt0` between devices.
  Run `ip address show wt0` on the one device and copy the IP address.
  Then run `ping $IP_ADDRESS` on the other device, with `$IP_ADDRESS` replaced with the IP address.


### Troubleshooting the ECUs

It happens relatively often that we look for problems in openDuT,
when the ECU or the wiring isn't working.

Here's some questions to help with that process:
* Is the ECU powered on?
* Is the ECU wired correctly?
* Is the ECU hooked up to the port of the edge device that you configured in openDuT?
* Does the setup work, if you replace openDuT with a physical wire?
