# SSL/TLS Certificate Expiry Runbook

## Severity: High

An expired SSL/TLS certificate causes all HTTPS connections to fail, making the application completely inaccessible to clients.

## Detection Methods

1. **Monitoring alerts**: Certificate expiry warning at 30 days, critical at 7 days
2. **Manual check**: `openssl s_client -connect <host>:443 -servername <host> 2>/dev/null | openssl x509 -noout -dates`
3. **Health checks**: External monitoring (e.g., Datadog, UptimeRobot) reports SSL errors
4. **Browser warnings**: Users see certificate warnings or connection refused errors
5. **Application errors**: `rustls` or `native-tls` handshake failures in application logs

## Response Procedure

### 1. Verify the Issue (2 min)

```bash
# Check certificate expiry
echo | openssl s_client -connect $HOST:443 -servername $HOST 2>/dev/null | openssl x509 -noout -dates -subject -issuer

# Check if the certificate is expired or about to expire
echo | openssl s_client -connect $HOST:443 -servername $HOST 2>/dev/null | openssl x509 -noout -checkend 2592000
# Returns 0 if cert expires in next 30 days (2592000 seconds)
```

### 2. Identify the Certificate Source

- **Reverse proxy** (nginx, Caddy, Traefik): Most common for production deployments
- **Load balancer** (AWS ALB, Cloudflare): Certificate managed at the LB level
- **Application-level**: Configured directly in the Rust server (unlikely for production)
- **Let's Encrypt**: Auto-renewed via certbot or ACME client

### 3. Renew the Certificate

#### Let's Encrypt (certbot)

```bash
# Force renewal
sudo certbot renew --force-renewal

# Verify renewal
sudo certbot certificates

# Reload the web server
sudo systemctl reload nginx  # or caddy, traefik
```

#### Manual Certificate (self-signed or purchased)

```bash
# Generate new self-signed certificate (development only)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=$HOST"

# Copy to the correct location
sudo cp cert.pem /etc/ssl/certs/tachyon.crt
sudo cp key.pem /etc/ssl/private/tachyon.key

# Reload the web server
sudo systemctl reload nginx
```

#### Cloud-managed (AWS ACM, Cloudflare)

- Use the cloud provider console or CLI to renew/reissue the certificate
- AWS: `aws acm request-certificate --domain-name $HOST --validation-method DNS`

### 4. Verify the Fix

```bash
# Check new certificate is active
echo | openssl s_client -connect $HOST:443 -servername $HOST 2>/dev/null | openssl x509 -noout -dates

# Verify the application is accessible
curl -s https://$HOST/health | jq '.status'

# Check no SSL errors
curl -v https://$HOST/health 2>&1 | grep -i "SSL\|certificate\|error"
```

### 5. Post-Incident

- Verify HTTPS is working correctly across all endpoints
- Check that WebSocket connections (wss://) are also working
- Update monitoring to alert earlier for future expirations
- Review and update auto-renewal configuration

## Prevention Measures

- Set up automated certificate renewal with certbot timer/ cron:
  ```
  0 0,12 * * * certbot renew --quiet --deploy-hook "systemctl reload nginx"
  ```
- Configure monitoring alerts at 30, 14, and 7 days before expiry
- Use ACME DNS-01 validation for wildcard certificates
- Test the renewal process monthly in staging
- Document certificate locations and renewal procedures
- Consider using a managed certificate service (Cloudflare, AWS ACM) to eliminate manual renewal
