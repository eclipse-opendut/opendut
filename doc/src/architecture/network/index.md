# Network

This chapter explains how openDuT securely connects devices across different networks by tunneling ethernet traffic, detailing the encapsulation, encryption, and MTU considerations that ensure reliable and transparent communication.

## Ethernet

OpenDuT uses GRE (Generic Routing Encapsulation) to securely tunnel ethernet traffic between devices. 
This enables devices to communicate over the openDuT network as if they were on the same local network, regardless of their physical location. 
GRE encapsulation also supports multiple VLANs, allowing for flexible network configurations. To protect data from third-party observation, GRE traffic is encrypted using WireGuard.
To prevent packet loss from the device under test (DuT), the MTU (Maximum Transmission Unit) of the WireGuard tunnel is increased to 1542 bytes. 
This accommodates a standard Ethernet frame size of 1514 bytes, an additional VLAN tag (4 bytes) and the GRE overhead (24 bytes).
Packet fragmentation may occur at the edge when the DuT sends packets with the full standard MTU of 1500 bytes. 
This is expected as we do not modify the DuT or its software, which may not handle reduced MTU values well.
The network stack handles this efficiently, and any increase in latency is typically minimal, ensuring reliable communication across the openDuT network.

### MTU considerations

The MTU size of the WireGuard tunnel is set to 1542 bytes (allows single VLAN tag), which is larger than the standard Ethernet MTU of 1500 bytes.
If devices use stacked VLANs (QinQ, 802.1ad), the MTU should be set to 1546 bytes to allow for the extra VLAN tag.

### Conclusion

In summary, openDuT’s approach enables seamless connectivity between devices that would otherwise be unable to communicate across different networks. 
By tunneling Ethernet traffic and accommodating standard MTU sizes, devices do not require any modification or special configuration, even when some software cannot handle reduced MTU values. 
While minor packet fragmentation may occur, the advantages of secure, flexible, and transparent networking far outweigh this drawback, making openDuT the optimal solution for reliable device integration.
