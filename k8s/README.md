# Installing Containerd on Multiple Nodes Using Ansible

This guide explains how to set up and use Ansible to install `containerd` on multiple nodes. It includes instructions for creating and using the Ansible installation script, the inventory file, and the Ansible playbook.

## Prerequisites

- A control machine with Ansible installed
  - Linux
  - [OSX](ansibleOSX/README.md)
- SSH access to all target nodes from the control machine.
- Target nodes running Ubuntu.

## 1. **Inventory File (`hosts.ini`)**

   Create an inventory file named `hosts.ini` to list your target nodes.

   ```ini
   [containerd_nodes]
   node1 ansible_host=192.168.1.1 ansible_user=ubuntu
   node2 ansible_host=192.168.1.2 ansible_user=ubuntu
   node3 ansible_host=192.168.1.3 ansible_user=ubuntu
   node4 ansible_host=192.168.1.4 ansible_user=ubuntu
   node5 ansible_host=192.168.1.5 ansible_user=ubuntu
```
   
## 2. Install Ansible on the Control Machine

For Linux, run `ansibleInstall.sh` on OSX run `setup_ansible_and_push_keys.sh` 

## 3. Execute Ansible Containerd Installation.
`ansible-playbook -i hosts.ini containerd.yml`

