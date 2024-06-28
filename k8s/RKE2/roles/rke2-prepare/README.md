# RKE2 Prepare Role

This role is responsible for preparing the RKE2 configuration and systemd service files on both server and agent nodes.

## Role Structure

```
rke2-prepare/
├── tasks
│   └── main.yaml
└── templates
    ├── rke2-agent.service.j2
    ├── rke2-server-config.j2
    └── rke2-server.service.j2
```

## Tasks

The main tasks performed by this role (as defined in `tasks/main.yaml`) include:

1. Creating necessary directories for RKE2 configuration and token
2. Deploying RKE2 server configuration
3. Creating systemd service files for RKE2 server and agent
4. Enabling and starting the RKE2 server on the first server node
5. Handling the node token (waiting for it, fetching it, and setting permissions)
6. Setting up kubectl for the user
7. Configuring the kubeconfig file

## Templates

1. `rke2-agent.service.j2`: Systemd service file for RKE2 agent
2. `rke2-server-config.j2`: Configuration file for RKE2 server
3. `rke2-server.service.j2`: Systemd service file for RKE2 server

## Variables

This role uses variables defined in `inventory/group_vars/all.yaml`, including:

- `vip`: The Virtual IP address managed by pfSense for load balancing
- `cluster_cidr`: Custom CIDR for cluster network
- `service_cidr`: Custom CIDR for service network

## Usage

This role should be applied to all nodes in the cluster, with specific tasks conditionally executed for server or agent nodes.

## Notes

- The RKE2 server is only enabled and started on the first server node.
- The node token is fetched from the first server node and made available to other nodes.
- kubectl is set up for use on the first server node.
- The kubeconfig file is copied and modified for user access on the first server node.

## Potential Improvements

1. Implement error handling for cases where the node token or kubectl isn't available within the expected timeframe.
2. Consider parameterizing more values in the templates, such as the Kubernetes domain name.
3. Add tasks to verify the successful setup of RKE2 on each node.
4. Implement logic to handle upgrades or reconfigurations of existing setups.