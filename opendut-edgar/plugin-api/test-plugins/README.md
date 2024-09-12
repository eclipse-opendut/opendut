# openDuT EDGAR Setup Plugins

This folder contains plugins for the EDGAR Setup.

### Build Distribution

Run `./build-distribution.sh` in the directory where the script is placed.

### Usage

To use the plugins, you need an EDGAR distribution already unpacked on a target machine.  
Create a `plugins/` folder in the EDGAR directory and unpack the distribution there.

Then create a file `plugins.txt` in the `plugins/` folder, where you reference the unpacked directory:  
```sh
echo "test-plugins/" >> plugins.txt
```
