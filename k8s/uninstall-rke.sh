#!/bin/bash

# Stop the RKE2 service
echo "Stopping RKE2 service..."
sudo systemctl stop rke2-server

# Disable the RKE2 service
echo "Disabling RKE2 service..."
sudo systemctl disable rke2-server

# Remove RKE2 directories and files
echo "Removing RKE2 directories and files..."
sudo rm -rf /etc/rancher/rke2 /var/lib/rancher/rke2 /var/lib/kubelet /var/lib/rancher

# Remove the RKE2 binary
echo "Removing RKE2 binary..."
sudo rm -f /usr/local/bin/rke2 /usr/local/bin/kubectl /usr/local/bin/crictl /usr/local/bin/containerd /usr/local/bin/ctr

# Remove additional configuration and data files
echo "Removing additional configuration and data files..."
sudo rm -rf /etc/systemd/system/rke2-server.service /etc/systemd/system/rke2-agent.service
sudo rm -rf /etc/systemd/system/multi-user.target.wants/rke2-server.service /etc/systemd/system/multi-user.target.wants/rke2-agent.service
sudo rm -rf /etc/systemd/system/sockets.target.wants/rke2d.sock

# Reload systemd and reset failed units
echo "Reloading systemd and resetting failed units..."
sudo systemctl daemon-reload
sudo systemctl reset-failed

# Check if any RKE2 processes are still running
echo "Checking for any remaining RKE2 processes..."
if ps aux | grep -q rke2; then
    echo "Warning: Some RKE2 processes are still running."
    ps aux | grep rke2
else
    echo "RKE2 uninstallation completed successfully."
fi
