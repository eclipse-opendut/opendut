# Set up the test environment

* in a virtual machine on [Linux](theo-setup-vm-linux.md)
* in a virtual machine on [Windows](theo-setup-vm-windows.md)
* on your host in [Docker](theo-setup-docker.md)

## Notes about the virtual machine

There are some important adaptions made to the virtual machine that you should be aware of.

> [!CAUTION]
> Within the VM the rust target directory `CARGO_TARGET_DIR` is overridden to `/home/vagrant/rust-target`.
> When running cargo within the VM, output will be placed in this directory!

This is done to avoid issues with hardlinks when cargo is run on a filesystem that does not support them (like vboxsf, the VirtualBox shared folder filesystem).
There is one exception to this. The distribution build with `cargo-ci` is placed in a subdirectory of the project, namely `target/ci/distribution`.
This should be fine since cargo-cross is building everything in docker anyway.

Furthermore, we use cicero for installing project dependencies like trunk and cargo-cross.
To avoid linking issues with binaries installed by cicero, we also set up a dedicated virtual environment for cicero in the VM.
This is done by overwriting the `CICERO_VENV_INSTALL_DIR` environment variable to `/home/vagrant/.cache/opendut/cicero/`.
