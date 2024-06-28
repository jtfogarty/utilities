# Add Server Role

This role is responsible for adding and configuring additional server nodes to an RKE2 Kubernetes cluster.

## Tasks

The main tasks performed by this role (as defined in `tasks/main.yaml`) are:

1. Deploy RKE2 server configuration to all servers except the first one.
2. Wait for the cluster API to be ready.
3. Ensure additional RKE2 servers are enabled and running.

## Templates

- `rke2-server-config.j2`: Template for RKE2 server configuration

This template configures:
- Kubeconfig write mode
- Cluster join token
- API server address
- TLS SAN entries (including VIP and all server IPs)
- Node labels

## Variables

Key variables used in this role:

- `vip`: Virtual IP address for the Kubernetes API server (defined in `group_vars/all.yaml`)
- `ansible_user`: The user for SSH connections and command execution
- `hostvars`: Ansible's host variables, used to access individual server details

## Usage

This role is typically included in the main playbook and runs on all server nodes except the first one. It configures additional servers to join the existing cluster.

## Notes

- This role assumes that the first server (server1) is already set up and running.
- The role uses the VIP (Virtual IP) configured in pfSense for load balancing.
- kube-vip related tasks have been removed as they are not needed with the pfSense setup.

## Dependencies

- Requires the first server to be properly initialized with RKE2.
- Depends on pfSense for load balancing instead of kube-vip.

## Configuration

Ensure that the following are properly configured:
1. The `vip` variable in `group_vars/all.yaml` matches your pfSense configuration.
2. pfSense HAProxy is set up to forward traffic to all server nodes on port 6443.
3. Network allows traffic from Kubernetes nodes to the VIP address for API server communication.