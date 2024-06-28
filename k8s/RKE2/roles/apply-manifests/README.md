# Apply Manifests Role

This role is responsible for applying various manifests and configurations to the Kubernetes cluster after it has been set up with RKE2.

## Tasks

The main tasks performed by this role are:

1. Wait for Kubernetes nodes to be ready
2. Apply MetalLB namespace
3. Apply MetalLB manifest
4. Wait for MetalLB pods to be ready
5. Apply MetalLB L2 Advertisement
6. Configure MetalLB IP address pool
7. Create namespace for Rancher
8. Install cert-manager CRDs
9. Create cert-manager namespace
10. Deploy cert-manager
11. Deploy Rancher

## Recent Changes

1. Updated MetalLB version variable usage:
   - Now using `{{ metallb_version }}` for both the namespace and main manifest URLs.

2. Added cert-manager version variable:
   - New variable `cert_manager_version` in `group_vars/all.yaml`
   - Updated cert-manager CRD installation task to use this variable

3. VIP (Virtual IP) update:
   - Changed from 10.10.100.4 to 10.10.100.2 in `group_vars/all.yaml`
   - This change aligns with the HAProxy configuration in pfSense

## Templates

- `metallb-ippool.j2`: Template for MetalLB IP address pool configuration

## Variables

Key variables used in this role (defined in `group_vars/all.yaml`):

- `metallb_version`: Version of MetalLB to install
- `lb_range`: IP range for MetalLB to use for load balancer services
- `lb_pool_name`: Name of the MetalLB address pool
- `cert_manager_version`: Version of cert-manager to install
- `vip`: Virtual IP address for the Kubernetes API server

## Notes

- Ensure that the `vip` variable is correctly used in all relevant configuration files.
- The current setup assumes SSL passthrough. If SSL offloading is required, additional configuration may be necessary.
- Health checks for the Kubernetes API server are configured in HAProxy using the `/healthz` endpoint.