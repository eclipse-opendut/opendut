# Setup

- Download the opendut-cleo binary for your target from the openDuT GitHub project: https://github.com/eclipse-opendut/opendut/releases
- Unpack the archive on your target system.

- EDGAR comes with a scripted setup, which you can initiate by running:  
```shell
opendut-edgar setup managed <SETUP-STRING>
```  
You can get the `<SETUP-STRING>` from LEA after creating a Peer.

This will configure your operating system and start the *EDGAR Service*, which will receive its configuration from *CARL*.
