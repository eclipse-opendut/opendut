# Commands

## Listing resources

To list resources you can decide whether to display the resources in a table or in JSON-format.
The default output format is a table which is displayed by not using the `--output` flag.
The `--output` flag is a global argument, so it can be used at any place in the command.

    opendut-cleo list --output=<format> <openDuT-resource>

## Creating resources

To create resources it depends on the type of resource whether an ID or connected devices have to be added to the command.

    opendut-cleo create <resource>

## Applying Configuration Files

To use configuration files, the resource topology can be written in a YAML format which can be applied with the following command:

    opendut-cleo apply <FILE_PATH>

The YAML file can look like this:

```yaml
---
version: v1
kind: PeerDescriptor
metadata:
  id: fc4f8da1-1d99-47e1-bbbb-34d0c5bf922a
  name: MyPeer
spec:
  location: Ulm
  network:
    interfaces:
    - id: 9a182365-47e8-49e3-9b8b-df4455a3a0f8
      name: eth0
      kind: ethernet
    - id: de7d7533-011a-4823-bc51-387a3518166c
      name: can0
      kind: can
      parameters:
        bitrate-kbps: 250
        sample-point: 0.8
        fd: true
        data-bitrate-kbps: 500
        data-sample-point: 0.8
  topology:
    devices:
    - id: d6cd3021-0d9f-423c-862e-f30b29438cbb
      name: ecu1
      description: ECU for controlling things.
      interface-id: 9a182365-47e8-49e3-9b8b-df4455a3a0f8
      tags:
        - ecu
        - automotive
    - id: fc699f09-1d32-48f4-8836-37e0a23cf794
      name: restbus-sim1
      description: Rest-Bus-Simulation for simulating other ECUs.
      interface-id: de7d7533-011a-4823-bc51-387a3518166c
      tags:
        - simulation
  executors:
    - id: da6ad5f7-ea45-4a11-aadf-4408bdb69e8e
      kind: container
      parameters:
        engine: podman
        name: nmap-scan
        image: debian
        volumes:
        - /etc/
        - /opt/
        devices:
        - ecu1
        - restbus-sim1
        envs:
        - name: VAR_NAME
          value: varValue
        ports:
        - 8080:8080
        command: nmap
        command-args:
        - -A
        - -T4
        - scanme.nmap.org
---
kind: ClusterDescriptor
version: v1
metadata:
  id: f90ffd64-ae3f-4ed4-8867-a48587733352
  name: MyCluster
spec:
  leader-id: fc4f8da1-1d99-47e1-bbbb-34d0c5bf922a
  devices:
    - d6cd3021-0d9f-423c-862e-f30b29438cbb
    - fc699f09-1d32-48f4-8836-37e0a23cf794

```

The `id` fields contain UUIDs. You can generate a random UUID when newly creating a resource with the `opendut-cleo create uuid` command.


## Generating PeerSetup Strings

To create a PeerSetup, it is necessary to provide the PeerID of the peer:

    opendut-cleo generate-setup-string <PeerID>

## Decoding PeerSetup Strings

If you have a peer setup string, and you want to analyze its content, you can use the `decode` command.  

    opendut-cleo decode-setup-string <String>

## Describing resources

To describe a resource, the ID of the resource has to be provided. The output can be displayed as text or JSON-format (`pretty-json` with line breaks or `json` without).

    opendut-cleo describe --output=<output format> <resource> --id

## Finding resources

You can search for resources by specifying a search criteria string with the `find` command. Wildcards such as `'*'` are also supported.

    opendut-cleo find <resource> "<at least one search criteria>"

## Delete resources

Specify the type of resource and its ID you want to delete in CARL.

    opendut-cleo delete <resource> --id <ID of resource>

# Usage Examples
## CAN Example
    # CREATE PEER
        opendut-cleo create peer --name "$NAME" --location "$NAME"

	# CREATE NETWORK INTERFACE
	    opendut-cleo create network-interface --peer-id "$PEER_ID" --type can --name vcan0

	# CREATE DEVICE
	    opendut-cleo create device --peer-id "$PEER_ID" --name device-"$NAME"-vcan0 --interface vcan0 

	# CREATE SETUP STRING
	    opendut-cleo generate-setup-string --id "$PEER_ID"

## Ethernet Example
    # CREATE PEER
        opendut-cleo create peer --name "$NAME" --location "$NAME"

	# CREATE NETWORK INTERFACE
	    opendut-cleo create network-interface --peer-id "$PEER_ID" --type eth --name eth0

	# CREATE DEVICE
	    opendut-cleo create device --peer-id "$PEER_ID" --name device-"$NAME"-eth0 --interface eth0 

	# CREATE SETUP STRING
	    opendut-cleo generate-setup-string --id "$PEER_ID"
