# RKE2 Cluster Setup and Rancher Deployment Playbook

This Ansible playbook automates the process of setting up an RKE2 (Rancher Kubernetes Engine 2) cluster and deploying Rancher. Here's a breakdown of its actions:

1. **Prepare Nodes**: 
   - Applies the [`prepare-nodes`](roles/prepare-nodes/README.md) role to all hosts in the inventory.
   - Involves system preparations like updating packages, configuring firewall rules, etc.

2. **Download RKE2 Binaries**:
   - Applies the [`rke2-download`](roles/rke2-download/README.md) role to all hosts.
   - Downloads the necessary RKE2 binaries on all nodes.

3. **Prepare RKE2 Configuration**:
   - Applies the [`rke2-prepare`](roles/rke2-prepare/README.md) role to all hosts.
   - Sets up RKE2 configuration files and systemd services.

4. **Configure RKE2 Server Nodes**:
   - Applies the [`add-server`](roles/add-server/README.md) role to hosts in the `server_nodes` group.
   - Sets up the RKE2 control plane nodes.

5. **Configure RKE2 Agent Nodes**:
   - Applies the [`add-agent`](roles/add-agent/README.md) role to hosts in the `agent_nodes` group.
   - Sets up the RKE2 worker nodes.

6. **Apply MetalLB and Rancher Manifests**:
   - Applies the [`apply-manifests`](roles/apply-manifests/README.md) role to `server_nodes`.
   - Configures MetalLB for load balancing in bare metal environments.
   - Deploys Rancher and its dependencies:
     a. Creates the `cattle-system` namespace for Rancher.
     b. Installs cert-manager CRDs (Custom Resource Definitions).
     c. Creates the `cert-manager` namespace.
     d. Deploys cert-manager using Helm.
     e. Deploys Rancher using Helm, setting the hostname and replica count.

## Project Structure

```
├── README.md (this file)
├── collections
│   └── requirements.yaml
├── inventory
│   ├── group_vars
│   │   └── all.yaml
│   └── hosts.ini
├── roles
│   ├── add-agent
│   ├── add-server
│   ├── apply-manifests
│   ├── prepare-nodes
│   ├── rke2-download
│   └── rke2-prepare
└── site.yaml
```

Each role has its own README file with detailed information about its tasks and variables.

## Notes:
- The playbook is configured to use pfSense for load balancing instead of kube-vip.
- There are areas for potential improvement, including support for different OS & architectures, multiple CNIs, improved wait logic, better use of Kubernetes Ansible plugins, optimized flow logic, and general clean-up.
- **Important**: Remember to review and modify variables in the `inventory/group_vars/all.yaml` file before running the playbook.

This playbook provides a comprehensive setup for an RKE2 cluster with Rancher management. It's designed to work with pfSense for load balancing but can be customized based on specific deployment needs.

For detailed information about each role, please refer to the linked README files above.