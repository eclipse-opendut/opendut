# Setup

### 1. Preparation

Make sure, you can reach CARL from your target system.  
For example, if CARL is hosted at `carl.opendut.local`, these two commands should work:
```sh
ping carl.opendut.local
curl https://carl.opendut.local
```

If you're self-hosting CARL, follow the instructions in [Self-Hosted Backend Server](#self-hosted-backend-server).

### 2. Download EDGAR

In the LEA web-UI, you can find a Downloads-menu in the sidebar.
You will then need to transfer EDGAR to your target system, e.g. via `scp`.

---

Alternatively, you can download directly to your target host with:
```sh
curl https://$CARL_HOST/api/edgar/$ARCH/download --output opendut-edgar.tar.gz
```
Replace `$CARL_HOST` with the domain where your CARL is hosted,  
and replace `$ARCH` with the appropriate CPU architecture.

Available CPU architectures are:
- x86_64-unknown-linux-gnu (most desktop PCs and server systems)
- armv7-unknown-linux-gnueabihf (Raspberry Pi)
- aarch64-unknown-linux-gnu (ARM64 systems)

### 3. Unpack the archive
Run this command to unpack EDGAR:
```sh
tar xf opendut-edgar.tar.gz
```

EDGAR should print version information, if you run:
```sh
opendut-edgar/opendut-edgar --version
```
If this throws an error, it is likely that you downloaded the wrong CPU architecture.

### 4. CAN Setup
If you want to use CAN, follow the steps in [CAN Setup](#can-setup) before continuing.

### 5. Scripted Setup

- EDGAR comes with a scripted setup, which you can initiate by running:  
  ```shell
  opendut-edgar setup managed <SETUP-STRING>
  ```  
You can get the `<SETUP-STRING>` from LEA or CLEO after creating a Peer.

This will configure your operating system and start the *EDGAR Service*, which will receive its configuration from *CARL*.


## CAN Setup
If you want to use CAN, it is mandatory to set the environment variable `OPENDUT_EDGAR_SERVICE_USER` as follows:
```shell
export OPENDUT_EDGAR_SERVICE_USER=root
```

When a cluster is deployed, EDGAR automatically creates a virtual CAN interface (by default: `br-vcan-opendut`) that is used as a bridge between Cannelloni instances and physical CAN interfaces. EDGAR automatically connects all CAN interfaces defined for the peer in CARL to this bridge interface. 

This also works with virtual CAN interfaces, so if you do not have a physical CAN interface and want to test the CAN functionality nevertheless, you can create a virtual CAN interface as follows. Afterwards, you will need to configure it for the peer in CARL.

```shell 
# Optionally, replace vcan0 with another name
ip link add dev vcan0 type vcan
ip link set dev vcan0 up
  ```

### Preparation
EDGAR relies on the Linux socketcan stack to perform local CAN routing and uses Cannelloni for CAN routing between EDGARs.
Therefore, we have some dependencies.
1. Install the following packages:
  ```shell
  sudo apt install -y can-utils
  ```
2. Download Cannelloni from here: https://github.com/eclipse-opendut/cannelloni/releases/
3. Unpack the Cannelloni tarball and copy the files into your filesystem like so:
  ```shell
  sudo cp libcannelloni-common.so.0 /lib/
  sudo cp libsctp.so* /lib/
  sudo cp cannelloni /usr/local/bin/
  ```

### Testing
When you configured everything and deployed the cluster, you can test the CAN connection between different EDGARs as follows:
- Execute on EDGAR leader, assuming the configured CAN interface on it is `can0`:
  ```shell
  candump -d can0
  ```
- On EDGAR peer execute (again, assuming can0 is configured here):
  ```shell
  cansend can0 01a#01020304
  ```
  Now you should see a can frame on leader side:
  ```text
  root@host:~# candump -d can0
  can0  01A   [4]  01 02 03 04
  ```

## Self-Hosted Backend Server

### DNS
If your backend server does not have a public DNS entry, you will need to adjust the `/etc/hosts/` file, by appending entries like this (using your server's IP address):
```
123.456.789.101 opendut.local
123.456.789.101 carl.opendut.local
123.456.789.101 auth.opendut.local
123.456.789.101 netbird.opendut.local
123.456.789.101 netbird-api.opendut.local
123.456.789.101 signal.opendut.local
123.456.789.101 nginx-webdav.opendut.local
```

Now the following command should complete without errors:
```
ping carl.opendut.local
```

### Self-Signed Certificate with Unmanaged Setup
If you plan to use the unmanaged setup and your NetBird server uses a self-signed certificate, follow these steps:

1. Create the certificate directory on the OLU: `mkdir -p /usr/local/share/ca-certificates/`

2. Copy your NetBird server certificate onto the OLU, for example, by running the following from outside the OLU:  
   ```sh
   scp certificate.crt root@10.10.4.1:/usr/local/share/ca-certificates/
   ```  
   Ensure that the certificate has a file extension of "crt".

3. Run `update-ca-certificates` on the OLU.  
   It should output "1 added", if everything works correctly.  

4. Now the following commands should complete without errors:
   ```
   curl https://netbird-api.opendut.local
   ```

## Troubleshooting
- In case of issues during the managed setup, see:
  ```shell
  less opendut-edgar/setup.log
  ```
  If the setup completed, but EDGAR does not show up as Healthy in LEA/CLEO, see:
  ```shell
  journalctl -u opendut-edgar
  ```
  For troubleshooting the VPN connection, you may also want to check the NetBird logs:
  ```shell
  cat /var/lib/netbird/client.log
  cat /var/lib/netbird/netbird.err
  cat /var/lib/netbird/netbird.out
  ```

- Sometimes it might be necessary to restart the EDGAR service:
  ```shell
  # Restart service
  sudo systemctl restart opendut-edgar
  # Check status
  systemctl status opendut-edgar
  ```

- It might happen that the NetBird Client started by EDGAR is not able to connect, in that case stop it and re-run EDGAR managed setup:
  ```shell
  sudo systemctl stop netbird
  ```

- EDGAR might start with an old IP, different from command `sudo wg` would print. In that particular case
stop netbird service and opendut-edgar service and re-run the setup. This might happen to all
EDGARs. If this is not enough, and it keeps getting the old IP, it is necessary to set up all
devices and clusters from scratch.
  ```shell
  sudo wg
  ```

- If this error appears: `ERROR opendut_edgar::service::cannelloni_manager: Failure while invoking command line program 'cannelloni': 'No such file or directory (os error 2)'.`  
  Make sure, you've completed the [CAN Setup](#can-setup).
