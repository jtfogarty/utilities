#!/bin/bash

# Define the list of nodes and SSH user
nodes=(
    "k8s-rancher-01"
    "k8s-rancher-02"
    "k8s-rancher-03"
    "k8s-rancher-04"
    "k8s-rancher-05"
    "k8s-rancher-06"
    "k8s-rancher-07"
    "k8s-rancher-08"
    "k8s-rancher-09"
)
user="jtfogar"
ssh_key="~/.ssh/id_rsa_k8s"

# ANSI escape code for green text and red text
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to check for the token file with sudo
check_token_file() {
    local node=$1
    echo -e "${GREEN}$node${NC}"
    echo "Checking token file on $node..."
    ssh -i $ssh_key $user@$node "sudo ls /var/lib/rancher/rke2/server/node-token"
    if [ $? -eq 0 ]; then
        echo "Token file exists on $node"
    else
        echo -e "${RED}Token file does NOT exist on $node${NC}"
    fi
}

# Function to check journalctl logs for rke2-agent
check_journalctl_logs() {
    local node=$1
    echo "Checking journalctl logs for rke2-agent on $node..."
    ssh -i $ssh_key $user@$node "sudo journalctl -xeu rke2-agent.service | tail -n 20"
}

# Function to check the RKE2 config file with sudo
check_config_file() {
    local node=$1
    echo "Checking RKE2 config file on $node..."
    ssh -i $ssh_key $user@$node "sudo cat /etc/rancher/rke2/config.yaml || echo -e '${RED}Config file does NOT exist on $node${NC}'"
}

# Function to test network connectivity
test_network_connectivity() {
    local node=$1
    local control_plane_node=$2
    echo "Testing network connectivity to $control_plane_node from $node..."
    ssh -i $ssh_key $user@$node "curl -sk https://$control_plane_node:9345 || echo -e '${RED}Failed to connect to $control_plane_node:9345 from $node${NC}'"
    ssh -i $ssh_key $user@$node "curl -sk https://$control_plane_node:6443 || echo -e '${RED}Failed to connect to $control_plane_node:6443 from $node${NC}'"
}

# Check token file on node1
check_token_file "k8s-rancher-01"

# Check token files on node2 and node3
check_token_file "k8s-rancher-02"
check_token_file "k8s-rancher-03"

# Check journalctl logs for rke2-agent on all agent nodes
for node in "${nodes[@]:3}"; do
    check_journalctl_logs $node
done

# Check RKE2 config file on all nodes
for node in "${nodes[@]}"; do
    check_config_file $node
done

# Test network connectivity to the control plane nodes
control_plane_node="k8s-rancher-01"
for node in "${nodes[@]:3}"; do
    test_network_connectivity $node $control_plane_node
done
