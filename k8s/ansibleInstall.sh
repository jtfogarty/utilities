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
