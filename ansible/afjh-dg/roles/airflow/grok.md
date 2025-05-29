Thank you for the clarification! Since you're in an on-premises environment and want to focus on installing your company's SSL certificates (CA2.crt, Root certificate authority.crt, server-specific .crt, server-specific .pem, and .key file) on the single server running Airflow, I'll create an Ansible role to manage these certificates and update the Airflow configuration to reference them. This will avoid LDAP-related configurations and focus solely on certificate installation and HTTPS setup for the Airflow webserver using port **443** via an Nginx reverse proxy, as recommended in my previous response for security and best practices.

### Key Points and Assumptions
- **Certificates Provided**: You have:
  - `CA2.crt`: Intermediate CA certificate.
  - `Root certificate authority.crt`: Root CA certificate.
  - Server-specific `.crt` (e.g., `airflow-server.crt`): Server certificate.
  - Server-specific `.pem` (e.g., `airflow-server.pem`): Likely a duplicate of the server certificate or a combined certificate chain.
  - Server-specific `.key` (e.g., `airflow-server.key`): Private key for the server certificate.
- **Goal**: Install these certificates on the Airflow server, configure Nginx to use them for HTTPS on port **443**, and ensure Airflow’s webserver (running on **8080** internally) is accessible via HTTPS.
- **Environment**: On-premises, single server running Linux (RedHat-based, as per `ansible_os_family == 'RedHat'` in `airflow-main.yml`).
- **Port**: Use **443** for external HTTPS access via Nginx, with Airflow internally on **8080** (consistent with your original `af-web.j2` and `af-cfg.j2`).
- **Ansible Role**: Create a new role (`airflow-certificates`) to handle certificate installation, separate from the Airflow setup for modularity.
- **Airflow Configuration**: Update `airflow.cfg.j2` to reflect the HTTPS `base_url` on **443**, with Nginx handling SSL termination.
- **Previous Changes**: Assume the `Wants=postgresql.service` removal and remote database host updates from my previous responses are applied (e.g., `airflow_db_host` in `all.yaml`). If not, I’ll include relevant updates.
- **File Naming**: For clarity, I’ll assume the server-specific files are named `airflow-server.crt`, `airflow-server.pem`, and `airflow-server.key`. Adjust these in the Ansible variables if different.

### Solution Overview
1. **Ansible Role (`airflow-certificates`)**: 
   - Install the certificates in standard locations (e.g., `/etc/ssl/certs` for certificates, `/etc/ssl/private` for the key).
   - Combine the server certificate, intermediate CA, and root CA into a certificate chain file (e.g., `airflow-server-chain.crt`) for Nginx.
   - Ensure proper permissions (e.g., key readable by `airflow` group).
2. **Nginx Configuration**: 
   - Reuse the `nginx-config.yml` playbook and `nginx-airflow.conf.j2` from my previous response, updated to reference the certificate chain and key.
   - Proxy HTTPS traffic from **443** to Airflow on **8080**.
3. **Airflow Configuration**: 
   - Update `af-cfg.j2` to set `base_url = https://<server-hostname>:443`.
   - Keep `web_server_port = 8080` since Nginx handles SSL.
4. **Updated `all.yaml`**: 
   - Add variables for certificate paths and server hostname.
   - Ensure compatibility with existing Airflow settings.

---

### Updated and New Ansible Files

Below are the necessary files, organized as an Ansible role (`airflow-certificates`) and updates to existing files. All certificate paths are parameterized, and the setup assumes Nginx as the reverse proxy. Artifacts are wrapped in `<xaiArtifact>` tags with UUIDs, reusing UUIDs from previous responses where applicable and generating new ones for the role.

#### 1. Updated `all.yaml` (Add Certificate and Hostname Variables)

Update `all.yaml` to include variables for the certificate files, certificate chain, and server hostname. This assumes the remote database host is already set (e.g., `airflow_db_host`).


---
# Global settings
os: "linux"
arch: "amd64"
ansible_user: "ansible"
ansible_ssh_private_key_file: "~/.ssh/id_rsa"
ansible_python_interpreter: "/usr/bin/python3"

# PyEnv settings
pyenv_user: pyenv
pyenv_group: pyenv
pyenv_root: "/opt/pyenv"
pyenv_shims: "{{ pyenv_root }}/shims"
python_version: "3.9.16"

# Airflow settings
airflow_version: "2.9.3"
airflow_user: "airflow"
airflow_group: "airflow"
airflow_home: "/opt/airflow-{{ airflow_version }}"
airflow_symlink: "/opt/airflow"
venv_name: "airflow-{{ airflow_version }}"
airflow_config_path: "{{ airflow_home }}/airflow.cfg"
airflow_logs_path: "{{ airflow_home }}/logs"
airflow_dags_path: "{{ airflow_home }}/dags"
airflow_plugins_path: "{{ airflow_home }}/plugins"
airflow_db_host: "db.example.com"  # Replace with your remote DB host
airflow_sql_alchemy_conn: "postgresql+psycopg2://airflow:password@{{ airflow_db_host }}:5432/airflow"

# Certificate settings
airflow_server_hostname: "airflow.example.com"  # Replace with your server's hostname
airflow_cert_root_ca: "/etc/ssl/certs/root-ca.crt"
airflow_cert_intermediate_ca: "/etc/ssl/certs/ca2.crt"
airflow_cert_server_crt: "/etc/ssl/certs/airflow-server.crt"
airflow_cert_server_pem: "/etc/ssl/certs/airflow-server.pem"
airflow_cert_server_key: "/etc/ssl/private/airflow-server.key"
airflow_cert_chain: "/etc/ssl/certs/airflow-server-chain.crt"


**Changes**:
- Added `airflow_server_hostname` for the `base_url` and Nginx `server_name`.
- Added certificate paths (`airflow_cert_root_ca`, `airflow_cert_intermediate_ca`, etc.).
- Added `airflow_cert_chain` for the combined certificate chain used by Nginx.
- Kept existing Airflow and database settings, assuming `airflow_db_host` is set correctly.

#### 2. Updated `af-cfg.j2` (Use HTTPS on 443)

Update `airflow.cfg.j2` to set the `base_url` to **443** via the reverse proxy, using the parameterized hostname.


[core]
executor = SequentialExecutor
dags_folder = {{ airflow_dags_path }}
plugins_folder = {{ airflow_plugins_path }}
base_log_folder = {{ airflow_logs_path }}
sql_alchemy_conn = {{ airflow_sql_alchemy_conn }}
load_examples = False

[logging]
base_log_folder = {{ airflow_logs_path }}
logging_level = INFO

[webserver]
base_url = https://{{ airflow_server_hostname }}:443
web_server_port = 8080

[scheduler]
run_duration = -1
min_file_process_interval = 0
dag_dir_list_interval = 300


**Changes**:
- Changed `base_url` to `https://{{ airflow_server_hostname }}:443`.
- Kept `web_server_port = 8080` for Airflow’s internal listener.
- Removed `web_server_ssl_cert`, `web_server_ssl_key`, and `rbac` (since LDAP is not needed).

#### 3. New Ansible Role: `airflow-certificates`

Create a new Ansible role to manage certificate installation. The role will:
- Copy the root CA, intermediate CA, server certificate, server PEM, and server key to the server.
- Combine the server certificate, intermediate CA, and root CA into a chain file.
- Set appropriate permissions.

**Directory Structure**:
```
roles/
  airflow-certificates/
    tasks/
      main.yml
    files/
      root-ca.crt
      ca2.crt
      airflow-server.crt
      airflow-server.pem
      airflow-server.key
```

##### `roles/airflow-certificates/tasks/main.yml`

This defines the tasks to install and combine certificates.


---
- name: Ensure SSL certificate directory exists
  ansible.builtin.file:
    path: "/etc/ssl/certs"
    state: directory
    owner: root
    group: root
    mode: '0755'

- name: Ensure SSL private key directory exists
  ansible.builtin.file:
    path: "/etc/ssl/private"
    state: directory
    owner: root
    group: "{{ airflow_group }}"
    mode: '0750'

- name: Copy root CA certificate
  ansible.builtin.copy:
    src: "root-ca.crt"
    dest: "{{ airflow_cert_root_ca }}"
    owner: root
    group: root
    mode: '0644'

- name: Copy intermediate CA certificate
  ansible.builtin.copy:
    src: "ca2.crt"
    dest: "{{ airflow_cert_intermediate_ca }}"
    owner: root
    group: root
    mode: '0644'

- name: Copy server certificate
  ansible.builtin.copy:
    src: "airflow-server.crt"
    dest: "{{ airflow_cert_server_crt }}"
    owner: root
    group: root
    mode: '0644'

- name: Copy server PEM certificate
  ansible.builtin.copy:
    src: "airflow-server.pem"
    dest: "{{ airflow_cert_server_pem }}"
    owner: root
    group: root
    mode: '0644'

- name: Copy server private key
  ansible.builtin.copy:
    src: "airflow-server.key"
    dest: "{{ airflow_cert_server_key }}"
    owner: root
    group: "{{ airflow_group }}"
    mode: '0640'

- name: Create certificate chain
  ansible.builtin.shell:
    cmd: cat {{ airflow_cert_server_crt }} {{ airflow_cert_intermediate_ca }} {{ airflow_cert_root_ca }} > {{ airflow_cert_chain }}
    creates: "{{ airflow_cert_chain }}"
  changed_when: true


**Notes**:
- Certificates are stored in `roles/airflow-certificates/files/` on the Ansible control node.
- The chain file combines `airflow-server.crt`, `ca2.crt`, and `root-ca.crt` in that order (server cert first, then intermediates, then root).
- The private key is readable by the `airflow` group, as Nginx may run as a user in this group (adjusted below).

##### Certificate Files

Place the following files in `roles/airflow-certificates/files/`:
- `root-ca.crt`
- `ca2.crt`
- `airflow-server.crt`
- `airflow-server.pem`
- `airflow-server.key`

If your files have different names, update the `src` fields in `main.yml` or rename the files to match.

#### 4. Updated `nginx-config.yml` (Use Certificate Chain)

Update the Nginx playbook to use the certificate chain and private key. This is similar to my previous response but tailored to the new certificate variables.


---
- name: Configure Nginx as reverse proxy for Airflow
  hosts: airflow
  become: yes
  vars_files:
    - all.yaml
  tasks:
    - name: Install Nginx
      ansible.builtin.dnf:
        name: nginx
        state: present
        update_cache: true
      when: ansible_os_family == 'RedHat'

    - name: Configure Nginx for Airflow
      ansible.builtin.template:
        src: "nginx-airflow.conf.j2"
        dest: "/etc/nginx/conf.d/airflow.conf"
        owner: root
        group: root
        mode: '0644'
      notify: Reload Nginx

    - name: Ensure Nginx is enabled and started
      ansible.builtin.systemd:
        name: nginx
        enabled: true
        state: started

  handlers:
    - name: Reload Nginx
      ansible.builtin.systemd:
        name: nginx
        state: reloaded


**Changes**:
- Removed certificate copy tasks, as they’re handled by the `airflow-certificates` role.
- Kept the Nginx configuration template and service management.

#### 5. Updated `nginx-airflow.conf.j2` (Reference Certificate Chain)

Update the Nginx configuration to use the certificate chain and private key.


server {
    listen 443 ssl;
    server_name {{ airflow_server_hostname }};

    ssl_certificate {{ airflow_cert_chain }};
    ssl_certificate_key {{ airflow_cert_server_key }};
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name {{ airflow_server_hostname }};
    return 301 https://$host$request_uri;
}


**Changes**:
- Changed `server_name` to `{{ airflow_server_hostname }}`.
- Set `ssl_certificate` to `{{ airflow_cert_chain }}`.
- Set `ssl_certificate_key` to `{{ airflow_cert_server_key }}`.

---

### How to Apply These Changes

1. **Prepare Certificates**:
   - Place `root-ca.crt`, `ca2.crt`, `airflow-server.crt`, `airflow-server.pem`, and `airflow-server.key` in `roles/airflow-certificates/files/`.
   - If your certificate files have different names, update the `src` fields in `roles/airflow-certificates/tasks/main.yml` or rename the files.

2. **Update Existing Files**:
   - Replace `all.yaml`, `af-cfg.j2`, `nginx-config.yml`, and `nginx-airflow.conf.j2` with the versions above.
   - Ensure `af-web.j2` and `af-svr.j2` are updated to remove `Wants=postgresql.service` (as per my previous response).

3. **Create the `airflow-certificates` Role**:
   - Create the directory structure `roles/airflow-certificates/`.
   - Add `tasks/main.yml` and the certificate files in `files/`.

4. **Run Playbooks**:
   - Run the certificate role to install certificates:
     ```bash
     ansible-playbook -i inventory.yml -e "role=airflow-certificates" site.yml
     ```
     (Assuming a `site.yml` playbook like below.)
   - Run the main Airflow playbook to update `airflow.cfg`:
     ```bash
     ansible-playbook airflow-main.yml
     ```
   - Run the Nginx playbook to configure the reverse proxy:
     ```bash
     ansible-playbook nginx-config.yml
     ```

5. **Sample `site.yml` for Role Execution**:
   Create a `site.yml` to apply the `airflow-certificates` role:
   ```yaml
   ---
   - hosts: airflow
     roles:
       - "{{ role }}"
   ```

6. **Verify**:
   - Access Airflow at `https://<your-server-hostname>` (e.g., `https://airflow.example.com`).
   - Check Nginx logs (`/var/log/nginx/error.log`) for certificate issues.
   - Verify Airflow logs (`/opt/airflow-2.9.3/logs`) for connectivity.
   - Ensure your browser trusts the certificate (the root and intermediate CAs should be in the client’s trust store).

7. **Firewall**:
   - Open ports **443** and **80**:
     ```bash
     sudo firewall-cmd --add-port=443/tcp --permanent
     sudo firewall-cmd --add-port=80/tcp --permanent
     sudo firewall-cmd --reload
     ```

---

### Notes and Assumptions

- **Certificate Files**:
  - The `.pem` file is assumed to be a duplicate of the server certificate or a chain. If it’s not needed, you can skip copying it by removing the task in `main.yml`.
  - The chain file (`airflow-server-chain.crt`) combines certificates in order: server, intermediate, root. This is standard for Nginx ([Nginx SSL Configuration](https://nginx.org/en/docs/http/configuring_https_servers.html)).
  - If your CA certificates are already trusted by the system, you may only need the server certificate and key in the chain. Adjust the `cat` command in `main.yml` accordingly.

- **Permissions**:
  - The private key is readable by the `airflow` group, assuming Nginx runs as a user in this group (common in RedHat with `nginx` user in a shared group). If Nginx uses a different group, update the `group` in `main.yml`.

- **Server Hostname**:
  - Replace `airflow.example.com` in `all.yaml` with your server’s actual hostname, matching the server certificate’s Common Name (CN) or Subject Alternative Name (SAN).

- **Nginx User**:
  - If Nginx runs as a non-root user not in the `airflow` group, you may need to adjust permissions or use `sudo` rules. Check with `ps aux | grep nginx`.

- **Existing Setup**:
  - Assumes `af-web.j2` and `af-svr.j2` are updated to remove `Wants=postgresql.service` and `airflow_db_host` is set correctly in `all.yaml`.
  - If you need these updates included, let me know.

- **Certificate Trust**:
  - Ensure clients (browsers, API callers) trust your company’s root and intermediate CAs. You may need to distribute `root-ca.crt` and `ca2.crt` to client trust stores.

---

This solution provides a modular Ansible role (`airflow-certificates`) to install your company’s SSL certificates and configures Nginx to serve Airflow over HTTPS on **443** in your on-premises environment. If you have specific certificate filenames, a different hostname, or additional requirements (e.g., direct **443** binding without Nginx, specific Nginx configurations), please provide details, and I’ll adjust the playbooks accordingly!