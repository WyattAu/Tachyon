# WebSocket Connection Storm Runbook

## Severity: High

A WebSocket storm is a sudden influx of WebSocket connections that overwhelms the server's connection handling capacity, causing resource exhaustion and degrading service for all users.

## Detection Methods

1. **Connection count spike**: `websocket_connections_active` metric surges beyond normal baseline
2. **Memory growth**: Server memory usage increases proportionally with connection count
3. **CPU spike**: High CPU from connection establishment and message broadcasting
4. **Client errors**: Increased WebSocket disconnections or connection refused errors
5. **Application logs**: Connection timeout or accept errors

## Response Procedure

### 1. Assess the Situation (5 min)

```bash
# Check WebSocket connection count (via Prometheus)
curl -s http://localhost:8080/metrics/prometheus | grep websocket

# Check active connections at the OS level
ss -s | grep -i "estab"
ss -tn | grep ":8080" | wc -l

# Check server resource usage
ps aux | grep tachyon-server
free -h
top -bn1 | head -5
```

### 2. Identify the Source

- **Legitimate traffic spike**: Sudden increase in real users (e.g., after a marketing push)
- **Bot/crawler**: Automated tool opening many connections
- **Client bug**: Frontend application reconnecting in a loop due to a bug
- **Reconnection storm**: Network instability causing mass reconnects

```bash
# Check connection distribution by IP
ss -tn state established '( dport = :8080 )' | awk '{print $5}' | cut -d: -f1 | sort | uniq -c | sort -rn | head -20

# Check for rapid reconnect patterns in logs
grep -i "websocket\|upgrade\|connect" /var/log/tachyon/app.log | tail -100
```

### 3. Mitigate

#### Immediate: Rate Limit or Block

```bash
# Block specific IPs if identified as malicious
sudo iptables -A INPUT -s <malicious_ip> -p tcp --dport 8080 -j REJECT

# Or limit connections per IP using iptables
sudo iptables -A INPUT -p tcp --dport 8080 -m connlimit --connlimit-above 50 -j REJECT
```

#### Application-Level: Enable Connection Limits

- The rate limit middleware (`middleware/rate_limit.rs`) can be configured to limit WebSocket upgrade requests
- Review and tighten `max_connections` in the server configuration

#### Load Balancer: Shed Connections

If behind a load balancer:
- Enable connection draining to gracefully close connections
- Scale up the number of server instances
- Set a maximum connection limit at the LB level

### 4. Stabilize

```bash
# If the server is unresponsive, restart it
systemctl restart tachyon-server

# After restart, verify health
curl -s http://localhost:8080/health | jq '.status'

# Monitor connection count returning to normal
watch -n 5 'ss -tn state established "( dport = :8080 )" | wc -l'
```

### 5. Investigate Root Cause

- **Client bug**: Check if the frontend has an exponential backoff reconnect strategy
- **Missing backpressure**: Verify the server sends pause frames when overwhelmed
- **Broadcast storms**: Check if a single document/channel is causing excessive message broadcasting
- **Configuration**: Review WebSocket idle timeout and max connection settings

### 6. Post-Incident

- Implement WebSocket connection rate limiting per IP
- Add circuit breakers for broadcast operations
- Implement connection quotas per user
- Add automated scaling triggers based on WebSocket connection count
- Fix any client-side reconnection bugs

## Prevention Measures

- Implement per-IP WebSocket connection limits
- Use exponential backoff with jitter for client reconnection
- Set reasonable idle timeouts (disconnect idle connections after 5-10 minutes)
- Implement graceful degradation: disable non-essential WebSocket features under load
- Configure horizontal auto-scaling based on connection count
- Monitor `websocket_connections_active` metric with alerting thresholds
- Load test WebSocket capacity to establish baseline limits
- Use a WebSocket-aware load balancer (sticky sessions or consistent hashing)
