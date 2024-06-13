
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

# Function to test a service on a node
test_service() {
    local node=$1
    local service=$2
    echo -e "${GREEN}$node${NC}"
    echo "Testing $service on $node..."
    ssh -i $ssh_key $user@$node "sudo systemctl is-active $service && sudo systemctl status $service | head -n 10"
    if [ $? -eq 0 ]; then
        echo "$service is running on $node"
    else
        echo -e "${RED}$service is NOT running on $node${NC}"
        if [ "$service" == "kubelet" ]; then
            find_kubelet_error $node
        fi
    fi
}

# Function to find an error for kubelet
find_kubelet_error() {
    local node=$1
    echo "Checking for kubelet errors on $node..."
    ssh -i $ssh_key $user@$node "sudo journalctl -u kubelet | tail -n 20"
}

# Function to check containerd socket
check_containerd_socket() {
    local node=$1
    echo "Checking containerd socket on $node..."
    ssh -i $ssh_key $user@$node "sudo ls -l /run/containerd/containerd.sock"
    if [ $? -eq 0 ]; then
        echo "Containerd socket is available on $node"
    else
        echo -e "${RED}Containerd socket is NOT available on $node${NC}"
    fi
}

# Function to test etcd
test_etcd() {
    local node=$1
    echo "Testing etcd on $node..."
    ssh -i $ssh_key $user@$node "ETCDCTL_API=3 etcdctl --endpoints=https://127.0.0.1:2379 endpoint health"
    if [ $? -eq 0 ]; then
        echo "etcd is healthy on $node"
    else
        echo -e "${RED}etcd is NOT healthy on $node${NC}"
    fi
    echo "Checking etcd service status on $node..."
    ssh -i $ssh_key $user@$node "sudo systemctl status etcd"
    echo "Checking etcd logs on $node..."
    ssh -i $ssh_key $user@$node "sudo journalctl -u etcd | tail -n 20"
    echo "Checking network connectivity to etcd on $node..."
    ssh -i $ssh_key $user@$node "curl -k https://127.0.0.1:2379/health"
}

# Check if an argument is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <service>"
    echo "Example: $0 containerd"
    exit 1
fi

# Iterate through each node and test the specified service
for node in "${nodes[@]}"; do
    test_service $node $1
    if [ "$1" == "containerd" ]; then
        check_containerd_socket $node
    fi
    test_etcd $node
done
