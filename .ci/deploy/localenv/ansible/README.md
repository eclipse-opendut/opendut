## How to use

1. Install Ansible on your machine, e.g on Debian-based systems:
   ```sh
   sudo apt install ansible
   ```

2. Define an inventory.yaml with parameters for your hosts, for example like so:
   ```
   backend:
     hosts:
       opendut-backend1:
         ip_for_edge_hosts_file: "123.456.789.101"
       opendut-backend2:
         ip_for_edge_hosts_file: "123.456.789.102"
     vars:
       repo_dir: "/data/opendut"

   edge:
     hosts:
       opendut-edge1:
         backend: opendut-backend1
         peer_id: "c1067a3a-6fd7-4466-96ef-56e1f51f778d"
       opendut-edge2:
         backend: opendut-backend1
         peer_id: "b4ade9ae-d2e4-46ac-84e5-2e7ef7aaca55"

   all:
     vars:
       ansible_user: "root"
   ```

3. Make sure you have entries in your SSH config for all the hosts declared in the inventory.yaml.

4. Run the scripts like so:
   ```sh
   ./playbook-backend.yaml -i inventory.yaml
   ./playbook-edge.yaml -i inventory.yaml
   ```
