## How to use

1. Install Ansible on your machine, e.g on Debian-based systems:
```sh
sudo apt install ansible
```

2. Define an inventory.ini with parameters for your hosts.

3. Make sure you have entries in your SSH config for all the hosts declared in the inventory.ini.

4. Run the script like so:
```sh
./playbook.yaml -i inventory.ini
```
