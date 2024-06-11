# Setup Ansible and Push SSH Keys to Nodes

This guide explains how to use the `setup_ansible_and_push_keys.sh` script to install Ansible on macOS and push SSH public keys to the nodes listed in an `ini` file.

## Prerequisites

- A macOS machine with internet access.
- SSH access to all target nodes from the macOS machine.

## Files Needed

1. **Inventory File (`hosts.ini`)**

   Create an inventory file named `hosts.ini` to list your target nodes.

   ```ini
   [containerd_nodes]
   node1 ansible_host=192.168.1.1 ansible_user=ubuntu
   node2 ansible_host=192.168.1.2 ansible_user=ubuntu
   node3 ansible_host=192.168.1.3 ansible_user=ubuntu
   node4 ansible_host=192.168.1.4 ansible_user=ubuntu
   node5 ansible_host=192.168.1.5 ansible_user=ubuntu

2. **setup file File**

```sh
chmod +x setup_ansible_and_push_keys.sh

./setup_ansible_and_push_keys.sh hosts.ini

./setup_ansible_and_push_keys.sh hosts.ini /path/to/your/ssh_key
```

