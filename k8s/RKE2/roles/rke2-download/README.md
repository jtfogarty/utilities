# RKE2 Download Role

This role is responsible for downloading and setting up the RKE2 binary on all nodes in the cluster.

## Role Structure

```
rke2-download/
└── tasks
    └── main.yaml
```

## Tasks

The main tasks performed by this role (as defined in `tasks/main.yaml`) are:

1. Create a directory for the RKE2 binary
2. Download the RKE2 binary
3. Set executable permissions on the RKE2 binary

## Variables

This role uses the following variables, which are defined in `inventory/group_vars/all.yaml`:

- `rke2_install_dir`: The directory where the RKE2 binary will be installed
- `rke2_binary_url`: The URL from which to download the RKE2 binary

These variables are centrally managed, allowing for consistent configuration across the entire playbook.

[Rest of the content remains the same...]

## Notes

- The RKE2 version is determined by the URL specified in `rke2_binary_url` in `inventory/group_vars/all.yaml`. Ensure this points to the desired version.
- This role does not have its own `vars/main.yaml` file, as all necessary variables are defined at the playbook level.
- This role assumes that the target systems have sufficient disk space for the RKE2 binary.

[Potential Improvements section remains the same]