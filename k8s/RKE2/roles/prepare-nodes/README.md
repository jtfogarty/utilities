# Prepare Nodes Role

This role is responsible for preparing the system configuration on all nodes (both servers and agents) before installing and configuring RKE2.

## Tasks

The main tasks performed by this role (as defined in `tasks/main.yaml`) are:

1. Enable IPv4 forwarding
2. Enable IPv6 forwarding

## Usage

This role should be applied to all nodes in the cluster (both servers and agents) as part of the initial setup process.

## Task Details

### Enable IPv4 forwarding

```yaml
- name: Enable IPv4 forwarding
  ansible.posix.sysctl:
    name: net.ipv4.ip_forward
    value: "1"
    state: present
    reload: true
  tags: sysctl
```

This task enables IPv4 forwarding, which is necessary for Kubernetes networking to function correctly.

### Enable IPv6 forwarding

```yaml
- name: Enable IPv6 forwarding
  ansible.posix.sysctl:
    name: net.ipv6.conf.all.forwarding
    value: "1"
    state: present
    reload: true
  tags: sysctl
```

This task enables IPv6 forwarding. While not all clusters use IPv6, enabling it ensures compatibility if IPv6 is used in the future.

## Dependencies

- Requires the `ansible.posix` collection for the `sysctl` module.

## Notes

- These configurations are persistent across reboots.
- The `reload: true` option ensures that the changes take effect immediately.
- The `sysctl` tag allows for selective execution of these tasks if needed.

## Potential Improvements

1. Add additional system preparations such as:
   - Disabling swap
   - Setting up required kernel modules
   - Configuring firewall rules
2. Add checks to verify the changes have been applied successfully.
3. Consider making IPv6 forwarding optional based on a variable, in case IPv6 is not used in some environments.