#!/usr/bin/env bash

echo "All containers started. You may observe the containers by connecting to the VM:"
echo "vagrant ssh opendut-vm"

echo -e "\n---------------------\n"
echo "docker ps"
echo "cd /vagrant"
echo "docker compose logs --tail=0 --follow"

echo -e "\n---------------------\n"
echo "Open OpenDuT Browser at: http://192.168.56.10:3000"
