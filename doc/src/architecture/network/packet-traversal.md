# Packet traversal

The following image depicts a ICMP packet traversing from ECU 1 to ECU 2, passing through the edge devices and the WireGuard tunnel. 
The packet is fragmented due to the MTU size of the WireGuard tunnel, which is set to 1542 bytes at the WireGuard interface `wt0`.

## Packet Traversal Example


![Packet traversal](img/openDuT-fragmentation.drawio.svg)

## Packet Traversal Table

The following table shows the packet traversal from ECU 1 to ECU 2, including the time, source and destination IP addresses, protocol, frame size, interface names and additional information about the packets. 
This example illustrates how the ICMP echo request is fragmented and reassembled as it traverses through the network interfaces and edge devices.

| #  | Time     | Source      | Destination | Protocol  | Size | Device                                | Interface ID | Interface name | Info                                                                       |
|----|----------|-------------|-------------|-----------|------|---------------------------------------|--------------|----------------|----------------------------------------------------------------------------|
| 1  | 2.547363 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1514 | <span style="color:blue">ECU 1</span> | 0            | vlan@eth0      | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            |
| 2  | 2.547365 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1518 | <span style="color:blue">ECU 1</span> | 1            | eth0           | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            |
| 3  | 2.547465 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1518 | edge1                                 | 2            | enx            | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            |
| 4  | 2.547465 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1518 | edge1                                 | 3            | br-opendut     | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            |
| 5  | 2.547521 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1518 | edge1                                 | 4            | gre            | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            | 
| 6  | 2.547555 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1542 | edge1                                 | 5            | wt0            | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            | 
| 7  | 2.547682 | 192.168.0.1 | 192.168.0.2 | IPv4      | 1514 | edge1                                 | 6            | end0           | Fragmented IP protocol (proto=UDP 17, off=0, ID=6ab0) [Reassembled in #8]  | 
| 8  | 2.547697 | 192.168.0.1 | 192.168.0.2 | WireGuard | 146  | edge1                                 | 6            | end0           | Transport Data, receiver=0x527A7C12, counter=277, datalen=1552             | 
| 9  | 2.547739 | 192.168.0.1 | 192.168.0.2 | IPv4      | 1514 | edge2                                 | 7            | end0           | Fragmented IP protocol (proto=UDP 17, off=0, ID=6ab0) [Reassembled in #10] | 
| 10 | 2.547739 | 192.168.0.1 | 192.168.0.2 | WireGuard | 146  | edge2                                 | 7            | end0           | Transport Data, receiver=0x527A7C12, counter=277, datalen=1552             |
| 11 | 2.547837 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1542 | edge2                                 | 8            | wt0            | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            |
| 12 | 2.547837 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1518 | edge2                                 | 9            | gre            | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            |
| 13 | 2.547837 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1518 | edge2                                 | 10           | br-opendut     | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            |
| 14 | 2.547896 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1518 | edge2                                 | 11           | enx            | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            | 
| 15 | 2.547896 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1518 | <span style="color:blue">ECU 2</span> | 12           | eth0           | Echo (ping) request  id=0x5285, seq=0/0, ttl=64                            | 
| 16 | 2.547896 | 10.0.0.1    | 10.0.0.2    | ICMP      | 1514 | <span style="color:blue">ECU 2</span> | 13           | vlan@eth0      | Echo (ping) request  id=0x5285, seq=0/0, ttl=64 (reply in 14)              | 
| 17 | 2.547959 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1514 | <span style="color:blue">ECU 2</span> | 13           | vlan@eth0      | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64 (request in 13)            | 
| 18 | 2.547959 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1518 | <span style="color:blue">ECU 2</span> | 12           | eth0           | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 19 | 2.547959 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1518 | edge2                                 | 11           | enx            | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 20 | 2.547959 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1518 | edge2                                 | 10           | br-opendut     | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 21 | 2.547969 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1518 | edge2                                 | 9            | gre            | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 22 | 2.547989 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1542 | edge2                                 | 8            | wt0            | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 23 | 2.548052 | 192.168.0.2 | 192.168.0.1 | IPv4      | 1514 | edge2                                 | 7            | end0           | Fragmented IP protocol (proto=UDP 17, off=0, ID=6507) [Reassembled in #24] | 
| 24 | 2.548060 | 192.168.0.2 | 192.168.0.1 | WireGuard | 146  | edge2                                 | 7            | end0           | Transport Data, receiver=0x010151F4, counter=110, datalen=1552             | 
| 25 | 2.548134 | 192.168.0.2 | 192.168.0.1 | IPv4      | 1514 | edge1                                 | 6            | end0           | Fragmented IP protocol (proto=UDP 17, off=0, ID=6507) [Reassembled in #26] | 
| 26 | 2.548134 | 192.168.0.2 | 192.168.0.1 | WireGuard | 146  | edge1                                 | 6            | end0           | Transport Data, receiver=0x010151F4, counter=110, datalen=1552             | 
| 27 | 2.548307 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1542 | edge1                                 | 5            | wt0            | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 28 | 2.548307 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1518 | edge1                                 | 4            | gre            | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 29 | 2.548307 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1518 | edge1                                 | 3            | br-opendut     | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 30 | 2.548388 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1518 | edge1                                 | 2            | enx            | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 31 | 2.548488 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1518 | <span style="color:blue">ECU 1</span> | 1            | eth0           | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 
| 32 | 2.548489 | 10.0.0.2    | 10.0.0.1    | ICMP      | 1514 | <span style="color:blue">ECU 1</span> | 0            | vlan@eth0      | Echo (ping) reply    id=0x5285, seq=0/0, ttl=64                            | 

Remarks:
- The interface ID corresponds to the numbers in the image above.
- The ICMP packet is sent from ECU 1 at `10.0.0.1` to ECU 2 at `10.0.0.2`.
- The packet is fragmented at the edge device due to the MTU size of the network interfaces that connect the edge devices. See steps 7-10 and 23-26.
- The packet request reaches the destination ECU 2 in step 16.

## Overhead analysis

The packet sent by the DuT has 1518 bytes, consisting of a standard ethernet frame with 1514 bytes and VLAN tag with 4 bytes.
This packet exceeds the MTU of 1500 bytes at the edge device.
In this case there are two packets sent between the edge devices with 146 bytes and 1514 bytes.
The overhead of OpenDuT can therefore be computed as follows: `(1514 + 146) - 1518 = 142 Bytes`

Breakdown:
* GRE encapsulation overhead: 24 bytes
* Fragmentation due to MTU limit (requires a second packet with IPv4 header and Ethernet frame): 34 bytes
* Encapsulation in another Ethernet frame (WireGuard tunnel): 34 bytes
* WireGuard encryption overhead: 1592 bytes (WireGuard packet) - 1542 bytes (original payload) = 50 bytes

Thus: 1660 - 50 - 34 - 34 - 24 = 1518 bytes
