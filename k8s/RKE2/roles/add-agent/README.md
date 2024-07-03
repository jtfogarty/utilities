# Add Agent Role

This role is responsible for adding and configuring agent nodes to an RKE2 Kubernetes cluster using a pfSense load balancer for high availability.

## Tasks

The main tasks performed by this role (as defined in `tasks/main.yaml`) are:

1. Deploy RKE2 Agent Configuration to all agent nodes.
2. Ensure RKE2 agents are enabled and running.

## Templates

- `rke2-agent-config.j2`: Template for RKE2 agent configuration

The template contains:
```yaml
write-kubeconfig-mode: "0644"
token: {{ hostvars[groups['server_nodes'][0]]['token'] }}
server: https://{{ vip }}:6443
node-label:
  - "agent=true"
```

## Variables

Key variables used in this role:

- `groups['agents']`: Ansible group containing all agent nodes
- `hostvars[groups['server_nodes'][0]]['token']`: The token used for agents to join the cluster
- `vip`: The Virtual IP address managed by pfSense for load balancing

## Usage

This role is typically included in the main playbook and runs on all nodes in the 'agents' group. It configures agent nodes to join the existing cluster through the pfSense load balancer.

## Notes

- This role assumes that the server nodes are already set up and running.
- The agent configuration uses the token from the first server node to join the cluster.
- Agents are labeled with "agent=true" for easy identification in the cluster.
- The configuration uses the VIP (Virtual IP) managed by pfSense instead of pointing to a specific server node, ensuring high availability.
- Agents connect to the Kubernetes API server on port 6443, which is the standard port load balanced by pfSense.

## Dependencies

- Requires the server nodes to be properly initialized with RKE2.
- Depends on the first server node ('server1') being correctly set up with a valid join token.
- Requires pfSense to be configured with the correct VIP and load balancing rules for the Kubernetes API server.

## Configuration

Ensure that the following are properly configured:
1. The 'agents' group in the Ansible inventory is correctly defined with all agent nodes.
2. The first server node ('server1') is properly initialized and has a valid join token.
3. The `vip` variable is correctly set to the Virtual IP address configured in pfSense.
4. pfSense is configured to load balance traffic to the Kubernetes API server (port 6443) across all server nodes.

## Potential Improvements

1. Implement logic to handle different tokens for different agents if required in the future.
2. Add health checks or verification steps to ensure agents have successfully joined the cluster.
3. Consider adding logic to handle upgrades or reconfigurations of existing agent nodes.

## Troubleshooting

If agents are unable to join the cluster:
1. Verify that the VIP is reachable from the agent nodes.
2. Check that pfSense is correctly load balancing traffic to all server nodes.
3. Ensure the join token is correct and hasn't expired.
4. Verify that the necessary ports (6443) are open in any firewalls between agents and the VIP.