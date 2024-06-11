# Installing Containerd on Multiple Nodes Using Ansible

This guide explains how to set up and use Ansible to install `containerd` on multiple nodes. It includes instructions for creating and using the Ansible installation script, the inventory file, and the Ansible playbook.

## Prerequisites

- A control machine with Ansible installed (Linux).
- SSH access to all target nodes from the control machine.
- Target nodes running Ubuntu.

## 1. Install Ansible on the Control Machine

### AnsibleInstall.sh

Create a script named `AnsibleInstall.sh` to install Ansible on the control machine.

```bash
#!/bin/bash

# Update and upgrade the system
sudo apt-get update -y
sudo apt-get upgrade -y

# Install required dependencies
sudo apt-get install -y software-properties-common

# Add Ansible PPA repository
sudo add-apt-repository --yes --update ppa:ansible/ansible

# Install Ansible
sudo apt-get update -y
sudo apt-get install -y ansible

# Verify Ansible installation
ansible --version

echo "Ansible installation completed successfully."

## 2. Execute Ansible Containerd Installation.
ansible-playbook -i hosts.ini containerd.yml

