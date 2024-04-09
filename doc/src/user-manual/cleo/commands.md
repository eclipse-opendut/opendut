# Commands

## Listing resources

To list resources you can decide whether to display the resources in a table or in JSON-format.
The default output format is a table which is displayed by not using the `--output` flag.

    opendut-cleo list --output=<format> <openDuT-resource>

## Creating resources

To create resources it depends on the type of resource whether an ID or connected devices have to be added to the command.

    opendut-cleo create <resource>

## Generating PeerSetup Strings

To create a PeerSetup, it is necessary to provide the PeerID of the peer:

    opendut-cleo generate-setup-key --id <PeerID>

## Decoding Setup Strings

If you have a setup string, and you want to analyze its content, you can use the `decode` command.  

    opendut-cleo decode --setup-string <String>

## Describing resources

To describe a resource, the ID of the resource has to be provided. The output can be displayed as text or JSON-format.

    opendut-cleo describe --output=<output format> <resource> --id 

## Finding resources

You can search for resources by specifying a search criteria string with the `find` command. Wildcards such as `'*'` are also supported.

    opendut-cleo find <resource> "<at least one search criteria>"

## Delete resources

Specify the type of resource and its ID you want to delete in CARL.

    opendut-cleo delete <resource> --id <ID of resource>
