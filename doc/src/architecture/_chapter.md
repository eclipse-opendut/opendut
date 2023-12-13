# Overview

![Overview.excalidraw.svg](..%2F..%2F..%2Fresources%2Fdiagrams%2FOverview.excalidraw.svg)

### Components
- **CARL** (Control And Registration Logic)
- **EDGAR** (Edge Device Global Access Router)
- **LEA** (Leasing ECU Access)
- **CLEO** (Command-Line ECU Orchestrator)
- **DUT** (Device under test)

# Functional description
The openDuT-Network provisions an end-to-end encrypted private network between **Devices under Test** (DuT), Test Execution Engines, RestBus simulations, and other devices.
To achieve this, openDuT uses **Edge Device Global Access Router** (EDGAR), which can tunnel the network traffic (Layer 2) of the connected devices into the openDuT-Network using **Generic Routing Encapsulation** (GRE). EDGAR registers with the **Control and Registration Logic** (CARL) and reports the type and status of its connected devices.
Multiple EDGARs can be linked to clusters via the graphical **Leasing ECU Access** (LEA) UI or the **Command-Line ECU Orchestrator** (CLEO) of CARL, and the openDuT-Network cluster can be provisioned for the user.

![openDuT_functional_diagram.svg](..%2F..%2F..%2Fresources%2Fdiagrams%2FopenDuT_functional_diagram.svg)

The openDuT-Network uses NetBird technology and provides its own NetBird server, including a TURN server in CARL and NetBird clients in the EDGARs. The NetBird clients of the clustered EDGARs automatically build a WireGuard network in star topology. If a direct connection between two EDGARs is not possible, the tunnel is routed through the TURN server in CARL.

![EDGAR_GRE_bridgeing.excalidraw.svg](..%2F..%2F..%2Fresources%2Fdiagrams%2FEDGAR_GRE_bridgeing.excalidraw.svg)

Within EDGAR, the openDUT-Bridge manages communication and routes outgoing packets to the GRE-Bridge(s). The GRE-Bridges encapsulate the packets and send them over fixed-assigned sources to fixed-assigned targets. When encapsulating, GRE writes the source and header information and the protocol type of the data packet into the GRE header of the packet. This offers the following advantages: different protocol types can be sent, network participants can be in the same subnet, and multiple VLANs can be transmitted through a single WireGuard tunnel.