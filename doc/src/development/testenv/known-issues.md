# Known Issues

## Copying data to and from the OpenDuT Browser

The OpenDuT Browser is a web browser running in a docker container. 
It is based on KasmVNC base image which allows containerized desktop applications from a web browser.
When using the OpenDuT Browser, you may want to copy data to and from the OpenDuT browser inside your own browser.
On Firefox this is restricted, and you may use the clipboard window on the left side of the OpenDuT Browser to copy data to your clipboard.

## Cargo Target Directory
When running cargo tasks within the virtual machine, you may see following error:
```
warning: hard linking files in the incremental compilation cache failed. copying files instead. consider moving the cache directory to a file system which supports hard linking in session dir
```
This is mitigated by setting a different target directory for cargo in `/home/vagrant/.bashrc` on the virtual machine:
```
export CARGO_TARGET_DIR=$HOME/rust-target
```

## Vagrant Permission Denied

Sometimes vagrant fails to insert the private key that was automatically generated.
This might cause this error (seen in git-bash on Windows):
```
$ vagrant ssh
vagrant@127.0.0.1: Permission denied (publickey).
```
This can be fixed by overwriting the vagrant-generated key with the one inserted during provisioning:
```sh
cp ~/.ssh/id_rsa .vagrant/machines/opendut-vm/virtualbox/private_key
```

## Vagrant Timeout
If the virtual machine is not allowed to create or use a private network it may cause a timeout during booting the virtual machine.

```
Timed out while waiting for the machine to boot. This means that
Vagrant was unable to communicate with the guest machine within
the configured ("config.vm.boot_timeout" value) time period.
```
* You may disable the private network by setting the environment variable `OPENDUT_DISABLE_PRIVATE_NETWORK=true` and explicitly halt and start the virtual machine again.
```shell
export OPENDUT_DISABLE_PRIVATE_NETWORK=true
vagrant halt
vagrant up
```

## Vagrant Custom Certificate Authority
When running behind an intercepting http proxy, you may run into issues with SSL certificate verification. 
```shell
ssl.SSLCertVerificationError: [SSL: CERTIFICATE_VERIFY_FAILED] certificate verify failed: self-signed certificate in certificate chain (_ssl.c:1007)
```

This can be mitigated by adding the custom certificate authority to the trust store of the virtual machine. 
* Place certificate authority file here: `resources/development/tls/custom-ca.crt`
* And re-run the provisioning of the virtual machine.
```shell
export CUSTOM_ROOT_CA=resources/development/tls/custom-ca.pem
vagrant provision
```

## Ctrl+C in Vagrant SSH
When using `cargo theo vagrant ssh` on Windows and pressing `Ctrl+C` to terminate a command, the ssh session may be closed.
