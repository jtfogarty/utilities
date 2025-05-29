I’ll help you create Ansible files to upload and install certificates on a Red Hat Linux server for your Airflow setup. Based on your requirements, you need to handle the following certificates:

- `CA2.crt` (company’s issuing CA certificate)
- `authority.crt` (root CA certificate)
- `authority.key` (root CA private key)
- `<server>.crt` (server-specific certificate for Airflow)
- `<server>.pem` (server-specific PEM file, assumed to be the private key or combined certificate/key)

You’ve specified a role directory called `certificates` with a `files` folder for certificates and a `tasks` folder for `main.yml`. I’ll outline the necessary Ansible files, including the role structure, and provide the `main.yml` task file to upload and install the certificates. I’ll also list any additional files needed and make assumptions where necessary, ensuring compatibility with Red Hat Linux.

### Assumptions
- The certificates will be stored in the `certificates/files/` directory within the role.
- The server-specific certificate and PEM files are named `airflow_server.crt` and `airflow_server.pem` (replace with actual names in your variables).
- Certificates will be installed to standard locations (e.g., `/etc/pki/tls/certs/` for certificates and `/etc/pki/tls/private/` for private keys).
- The root CA certificate (`authority.crt`) and issuing CA certificate (`CA2.crt`) need to be trusted by the system, which involves updating the system’s CA trust store.
- The `authority.key` and `airflow_server.pem` (if it’s a private key) require secure permissions (e.g., `0600`, owned by `root`).
- The Airflow server requires the server-specific certificate and key to be placed in a directory accessible to the Airflow process (e.g., `/etc/airflow/certs/`).
- You’re using a Red Hat-based system (e.g., RHEL, CentOS), so we’ll use `update-ca-trust` to manage trusted CAs.
- The `group_vars/all.yaml` file contains variables like certificate file names, destination paths, or Airflow user/group if needed.

### Role Structure
The role will be structured as follows:

```
roles/
  certificates/
    files/
      CA2.crt
      authority.crt
      authority.key
      airflow_server.crt
      airflow_server.pem
    tasks/
      main.yml
    templates/  (optional, not used here unless needed for configuration files)
    defaults/   (optional, for default variables)
      main.yml
```

### Files Needed
1. **Role Files**:
   - `roles/certificates/files/CA2.crt`: The issuing CA certificate.
   - `roles/certificates/files/authority.crt`: The root CA certificate.
   - `roles/certificates/files/authority.key`: The root CA private key.
   - `roles/certificates/files/airflow_server.crt`: The server-specific certificate.
   - `roles/certificates/files/airflow_server.pem`: The server-specific PEM file.
   - `roles/certificates/tasks/main.yml`: The main task file to upload and install certificates.
   - `roles/certificates/defaults/main.yml` (optional): Default variables for the role, in case `group_vars/all.yaml` doesn’t cover everything.

2. **Existing Files (Assumed)**:
   - `inventory`: Your inventory file defining the target Red Hat server(s).
   - `site.yml`: The playbook that calls the `certificates` role.
   - `group_vars/all.yaml`: Contains variables like certificate paths or Airflow-specific settings.

3. **Additional Files**:
   - No additional files are strictly required beyond the role structure, as templates or handlers are not needed for this straightforward task. However, I’ll include a `defaults/main.yml` to define default paths and permissions, which can be overridden in `group_vars/all.yaml`.

### Ansible Tasks Overview
The `main.yml` will:
- Create destination directories (e.g., `/etc/pki/tls/certs/`, `/etc/pki/tls/private/`, `/etc/airflow/certs/`).
- Upload the certificates and keys to the appropriate locations.
- Set correct ownership and permissions (e.g., `root:root`, `0644` for certificates, `0600` for private keys).
- Install the root CA (`authority.crt`) and issuing CA (`CA2.crt`) into the system’s CA trust store.
- Update the CA trust store using `update-ca-trust`.
- Ensure the Airflow-specific certificates are accessible to the Airflow process.

### Variables
I’ll define default variables in `roles/certificates/defaults/main.yml` for flexibility. You can override these in `group_vars/all.yaml` if needed.

### Ansible Files

#### 1. `roles/certificates/defaults/main.yml`
This file defines default variables for certificate paths, ownership, and permissions.

```yaml
# Default variables for the certificates role
cert_dir: /etc/pki/tls/certs
key_dir: /etc/pki/tls/private
airflow_cert_dir: /etc/airflow/certs
cert_owner: root
cert_group: root
cert_mode: "0644"
key_mode: "0600"
airflow_user: airflow
airflow_group: airflow

# Certificate file names (can be overridden in group_vars)
ca_cert_file: authority.crt
ca_key_file: authority.key
issuing_ca_file: CA2.crt
server_cert_file: airflow_server.crt
server_pem_file: airflow_server.pem
```

#### 2. `roles/certificates/tasks/main.yml`
This file contains the tasks to upload and install the certificates.

```yaml
---
- name: Ensure certificate directories exist
  ansible.builtin.file:
    path: "{{ item }}"
    state: directory
    owner: "{{ cert_owner }}"
    group: "{{ cert_group }}"
    mode: "0755"
  loop:
    - "{{ cert_dir }}"
    - "{{ key_dir }}"
    - "{{ airflow_cert_dir }}"

- name: Upload root CA certificate
  ansible.builtin.copy:
    src: "{{ ca_cert_file }}"
    dest: "{{ cert_dir }}/{{ ca_cert_file }}"
    owner: "{{ cert_owner }}"
    group: "{{ cert_group }}"
    mode: "{{ cert_mode }}"
  notify: Update CA trust

- name: Upload issuing CA certificate
  ansible.builtin.copy:
    src: "{{ issuing_ca_file }}"
    dest: "{{ cert_dir }}/{{ issuing_ca_file }}"
    owner: "{{ cert_owner }}"
    group: "{{ cert_group }}"
    mode: "{{ cert_mode }}"
  notify: Update CA trust

- name: Upload root CA private key
  ansible.builtin.copy:
    src: "{{ ca_key_file }}"
    dest: "{{ key_dir }}/{{ ca_key_file }}"
    owner: "{{ cert_owner }}"
    group: "{{ cert_group }}"
    mode: "{{ key_mode }}"

- name: Upload server certificate for Airflow
  ansible.builtin.copy:
    src: "{{ server_cert_file }}"
    dest: "{{ airflow_cert_dir }}/{{ server_cert_file }}"
    owner: "{{ airflow_user }}"
    group: "{{ airflow_group }}"
    mode: "{{ cert_mode }}"

- name: Upload server PEM file for Airflow
  ansible.builtin.copy:
    src: "{{ server_pem_file }}"
    dest: "{{ airflow_cert_dir }}/{{ server_pem_file }}"
    owner: "{{ airflow_user }}"
    group: "{{ airflow_group }}"
    mode: "{{ key_mode }}"

- name: Copy CA certificates to system trust store
  ansible.builtin.copy:
    src: "{{ cert_dir }}/{{ item }}"
    dest: /etc/pki/ca-trust/source/anchors/{{ item }}
    owner: "{{ cert_owner }}"
    group: "{{ cert_group }}"
    mode: "{{ cert_mode }}"
  loop:
    - "{{ ca_cert_file }}"
    - "{{ issuing_ca_file }}"
  notify: Update CA trust

- name: Ensure CA trust is updated
  ansible.builtin.meta: flush_handlers

# Handler to update CA trust
handlers:
  - name: Update CA trust
    ansible.builtin.command:
      cmd: update-ca-trust extract
    changed_when: true
```

### Explanation of Tasks
1. **Create Directories**: Ensures `/etc/pki/tls/certs`, `/etc/pki/tls/private`, and `/etc/airflow/certs` exist with appropriate permissions.
2. **Upload Certificates**:
   - Root CA (`authority.crt`) and issuing CA (`CA2.crt`) are uploaded to `/etc/pki/tls/certs`.
   - Root CA private key (`authority.key`) is uploaded to `/etc/pki/tls/private` with `0600` permissions.
   - Server certificate (`airflow_server.crt`) and PEM file (`airflow_server.pem`) are uploaded to `/etc/airflow/certs` with ownership set to the Airflow user/group.
3. **Trust CA Certificates**:
   - Copies `authority.crt` and `CA2.crt` to `/etc/pki/ca-trust/source/anchors/` to be included in the system’s trust store.
   - Triggers the `update-ca-trust extract` command via a handler to update the trust store.
4. **Handler**: Runs `update-ca-trust extract` only when CA certificates are modified.

### Additional Notes
- **Certificate Files**: Place `CA2.crt`, `authority.crt`, `authority.key`, `airflow_server.crt`, and `airflow_server.pem` in `roles/certificates/files/`. Ensure the filenames match those defined in `group_vars/all.yaml` or `defaults/main.yml`.
- **group_vars/all.yaml**: If you define variables like `server_cert_file` or `airflow_user` in `group_vars/all.yaml`, they’ll override the defaults. Example:
  ```yaml
  server_cert_file: my_airflow_server.crt
  server_pem_file: my_airflow_server.pem
  airflow_user: custom_airflow
  airflow_group: custom_airflow
  ```
- **site.yml**: Ensure your `site.yml` includes the `certificates` role, e.g.:
  ```yaml
  - hosts: airflow_servers
    roles:
      - certificates
  ```
- **Permissions**: The Airflow certificates are owned by the `airflow` user/group for accessibility. Adjust if Airflow runs as a different user.
- **Security**: Private keys (`authority.key`, `airflow_server.pem`) are set to `0600` to prevent unauthorized access.
- **Red Hat Compatibility**: The `update-ca-trust` command is specific to RHEL-based systems. If using a different distribution, you may need to modify the CA trust tasks.

### How to Use
1. Create the `certificates` role directory under `roles/`.
2. Place the certificate files in `roles/certificates/files/`.
3. Add the provided `defaults/main.yml` and `tasks/main.yml` to their respective directories.
4. Ensure your `site.yml` playbook applies the `certificates` role to the target hosts.
5. Run the playbook: `ansible-playbook -i inventory site.yml`.

If you need further customization (e.g., specific Airflow configuration files to use these certificates), let me know!