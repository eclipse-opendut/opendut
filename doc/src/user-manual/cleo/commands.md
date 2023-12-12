# Commands

## Listing resources

To list resources you can decide whether to display the resources in a table or in JSON-format.
The default output format is a table which is displayed by not using the `--output` flag.

    cleo list --output=<format> <openDuT-resource>

## Creating resources

To create resources it depends on the resource whether an ID or connected devices have to be added to the command.

    cleo create <resource>

## Generating PeerSetup Strings

To create a PeerSetup, providing the PeerID of the peer to be set up is necessary:

    cleo generate-setup-key --id <PeerID>

## Decoding Setup Strings

    cleo decode --setup-string <String>

## Describing resources

To describe a resource, their ID is to be provided. The output can be displayed via text or JSON-format.

    cargo cleo describe --output=<output format> <resource> --id 

## Finding resources

Wildcards such as '*' can be used to find resources.

    cleo find <resource> "<at least search criteria>"

## Delete resources

    cleo delete <resource> --id <ID of resource>
