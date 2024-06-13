#!/bin/bash

# Function to install Ansible on macOS
install_ansible() {
    echo "Installing Ansible on macOS..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    brew install ansible
    echo "Ansible installed successfully."
}

# Function to push SSH keys to nodes
push_ssh_keys() {
    local inventory_file=$1
    local ssh_key_file=$2

    # Generate SSH key pair if it doesn't exist
    if [ ! -f "$ssh_key_file" ]; then
        echo "Generating SSH key pair..."
        ssh-keygen -t rsa -b 4096 -f "$ssh_key_file" -N ""
    fi

    local public_key=$(cat "${ssh_key_file}.pub")

    # Read the inventory file and push the public key to each node
    echo "Pushing SSH public key to nodes..."
    while IFS= read -r line; do
        if [[ $line == *"ansible_host="* ]]; then
            node=$(echo $line | cut -d' ' -f1)
            host=$(echo $line | grep -oP 'ansible_host=\K\S+')
            user=$(echo $line | grep -oP 'ansible_user=\K\S+')
            echo "Pushing key to $node ($user@$host)..."
            ssh-copy-id -i "$ssh_key_file.pub" "$user@$host"
        fi
    done < "$inventory_file"
}

# Function to install RKE and kubectl on macOS
setup_rke_and_kubectl() {
    echo "Setting up RKE on macOS..."
    curl -LO https://github.com/rancher/rke/releases/download/v1.3.12/rke_darwin-amd64
    sudo mv rke_darwin-amd64 /usr/local/bin/rke
    sudo chmod +x /usr/local/bin/rke
    echo "RKE installed successfully."

    echo "Setting up kubectl on macOS..."
    curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/darwin/amd64/kubectl"
    chmod +x kubectl
    sudo mv kubectl /usr/local/bin/
    echo "kubectl installed successfully."
}

# Check if the inventory file is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <inventory_file> [ssh_key_file]"
    exit 1
fi

inventory_file=$1
ssh_key_file=${2:-~/.ssh/id_rsa}

# Install Ansible on macOS
install_ansible

# Push SSH keys to nodes
push_ssh_keys "$inventory_file" "$ssh_key_file"

# Setup RKE and kubectl on macOS
setup_rke_and_kubectl

echo "Setup complete."
