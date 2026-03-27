For a home lab setup like this (no public domain or external validation possible), you have two solid options for the SSL cert + key pair that Nginx needs for terminating HTTPS on the reverse proxy side. Salt-API itself runs on plain HTTP (typically `http://127.0.0.1:8080` or your configured port in `/etc/salt/master`), so Nginx will handle the HTTPS front-end and proxy the traffic internally.

**Recommended: Use `mkcert` (best for home labs)**  
This creates a local CA and issues certificates that are automatically trusted by your browser, curl, etc., on any machine where you install the CA. No annoying security warnings. It's perfect for internal services like Salt-API.

1. **Prerequisites** (on Debian/Ubuntu; adjust for your distro):  
   ```bash
   sudo apt update && sudo apt install libnss3-tools
   ```

2. **Install mkcert** (latest version, no manual version pinning needed):  
   ```bash
   curl -JLO "https://dl.filippo.io/mkcert/latest?for=linux/amd64"
   chmod +x mkcert-v*-linux-amd64
   sudo cp mkcert-v*-linux-amd64 /usr/local/bin/mkcert
   rm mkcert-v*-linux-amd64
   ```

3. **Install the local CA** (do this once; it adds the CA to your system trust store + Firefox/Chrome):  
   ```bash
   mkcert -install
   ```

4. **Generate the cert + key** (replace `saltapi.home.lab` with your actual hostname, FQDN, or IP(s) you'll use to access it):  
   ```bash
   sudo mkdir -p /etc/nginx/ssl
   cd /etc/nginx/ssl
   sudo mkcert -cert-file saltapi.crt -key-file saltapi.key \
     saltapi.home.lab "*.home.lab" localhost 127.0.0.1 ::1
   ```

   This outputs `saltapi.crt` and `saltapi.key` ready for Nginx.

**Quick fallback: Plain self-signed cert with OpenSSL** (no extra tools, but you'll get browser warnings unless you manually trust it)  
```bash
sudo mkdir -p /etc/nginx/ssl
sudo openssl req -x509 -nodes -days 365 -newkey rsa:4096 \
  -keyout /etc/nginx/ssl/saltapi.key \
  -out /etc/nginx/ssl/saltapi.crt \
  -subj "/CN=saltapi.home.lab"   # Change CN to your hostname
```

### Sample Nginx config
Create `/etc/nginx/sites-available/salt-api` (or drop it in your existing conf) with something like this:

```nginx
server {
    listen 80;
    server_name saltapi.home.lab;   # or your IP/hostname
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name saltapi.home.lab;   # must match the cert you generated

    ssl_certificate     /etc/nginx/ssl/saltapi.crt;
    ssl_certificate_key /etc/nginx/ssl/saltapi.key;

    # Recommended security headers (optional but nice)
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://127.0.0.1:8080;   # <-- your salt-api port (usually 8080)
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Salt-API can be chatty; increase timeouts if you see issues
        proxy_connect_timeout 90s;
        proxy_send_timeout 90s;
        proxy_read_timeout 90s;
    }
}
```

Then enable it and reload:
```bash
sudo ln -s /etc/nginx/sites-available/salt-api /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

### Quick tips
- In your Salt master config (`/etc/salt/master`), make sure the `rest_cherrypy` (or `rest_tornado`) section has `ssl: false` and the port matches what you're proxying to.
- Test with `curl -k https://saltapi.home.lab` (the `-k` skips verification for self-signed; omit if you used mkcert).
- If you're accessing from other machines, copy the CA from `mkcert -CAROOT` and import it there (or run `mkcert -install` on those machines too).
- For Salt clients (e.g., `salt-api` Python calls), point them at the new HTTPS URL and use `verify: false` or the CA bundle if needed.

This should get you up and running cleanly. If you hit any specific errors or share your current Nginx/salt-api config, I can refine it further!