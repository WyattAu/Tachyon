// WebSocket Reconnection Stress Test
// Tests: connection lifecycle, heartbeat, reconnect cycles, broadcast under load
// Usage: k6 run --execution-fragment 'scenarios: { ws: { executor: "constant-vus", vus: 20, duration: "2m" } }' load-tests/k6/WebSocketStress.js

import ws from 'k6/ws';
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'ws://localhost:8080';
const RECONNECT_CYCLES = parseInt(__ENV.RECONNECT_CYCLES || '10', 10);
const ROOM = __ENV.WS_ROOM || 'loadtest-room';
const HEARTBEAT_INTERVAL_MS = parseInt(__ENV.HEARTBEAT_INTERVAL_MS || '25000', 10);

const connectErrors = new Rate('ws_connect_errors');
const disconnectErrors = new Rate('ws_disconnect_errors');
const reconnectSuccess = new Rate('ws_reconnect_success');
const heartbeatMissed = new Rate('ws_heartbeat_missed');
const broadcastReceived = new Rate('ws_broadcast_received');
const connectionDuration = new Trend('ws_connection_duration');
const messageLatency = new Trend('ws_message_latency');

// k6 does not natively support WebSocket ping/pong at the application layer,
// so we use JSON text messages for heartbeat simulation.

function makeHeartbeatPayload() {
  return JSON.stringify({ type: 'pong', timestamp: Date.now() });
}

function makeBroadcastPayload(clientId) {
  return JSON.stringify({
    type: 'broadcast',
    channel: 'test-channel',
    sender: clientId,
    content: `Stress test message from ${clientId} at ${Date.now()}`,
  });
}

function stressConnection(clientId) {
  const url = `${BASE_URL}/ws/${ROOM}`;
  const startTime = Date.now();

  const res = ws.connect(url, null, function (socket) {
    socket.on('open', function () {
      connectionDuration.add(Date.now() - startTime);
    });

    socket.on('message', function (msg) {
      if (msg.includes('type')) {
        try {
          const parsed = JSON.parse(msg);
          if (parsed.type === 'ping' || parsed.type === 'pong') {
            // Respond to heartbeat pings from server
            socket.send(makeHeartbeatPayload());
          } else if (parsed.type === 'broadcast') {
            broadcastReceived.add(1);
            messageLatency.add(Date.now() - (parsed.timestamp || Date.now()));
          }
        } catch (e) {
          // Non-JSON message, ignore
        }
      }
    });

    socket.on('error', function (e) {
      connectErrors.add(1);
      console.error(`WS error [${clientId}]: ${e}`);
    });

    socket.setTimeout(function () {
      socket.send(makeBroadcastPayload(clientId));
    }, 1000);

    socket.setTimeout(function () {
      // Close after a short lifecycle to simulate reconnect
      socket.close();
    }, HEARTBEAT_INTERVAL_MS);
  });

  const connected = check(res, {
    'ws connected': (r) => r && r.status === 101,
  });
  reconnectSuccess.add(connected ? 1 : 0);
}

export const options = {
  scenarios: {
    ws: {
      executor: 'constant-vus',
      vus: 20,
      duration: '2m',
    },
  },
  thresholds: {
    ws_connect_errors: ['rate<0.05'],
    ws_disconnect_errors: ['rate<0.01'],
    ws_reconnect_success: ['rate>0.95'],
    ws_heartbeat_missed: ['rate<0.01'],
  },
};

export function setup() {
  // Verify server is reachable
  const healthUrl = BASE_URL.replace('ws://', 'http://').replace('wss://', 'https://');
  const res = http.get(`${healthUrl}/health`);
  if (res.status !== 200) {
    throw new Error(`Server not healthy: ${res.status}`);
  }
  console.log('WebSocket stress test: server is healthy');
  return { startTime: Date.now() };
}

export default function (data) {
  const clientId = `ws-stress-${__VU}-${Date.now()}`;
  try {
    stressConnection(clientId);
  } catch (e) {
    disconnectErrors.add(1);
    console.error(`VU ${__VU} connection failure: ${e.message}`);
  }
  sleep(Math.random() * 0.5 + 0.1);
}

export function handleSummary(data) {
  const connErr = data.metrics.ws_connect_errors;
  const discErr = data.metrics.ws_disconnect_errors;
  const reconnect = data.metrics.ws_reconnect_success;
  const hbMiss = data.metrics.ws_heartbeat_missed;
  const broadcast = data.metrics.ws_broadcast_received;
  const connDur = data.metrics.ws_connection_duration;

  let out = '\n=== WebSocket Stress Test Summary ===\n';
  out += `Connection errors: ${(connErr ? connErr.values.rate * 100 : 0).toFixed(2)}%\n`;
  out += `Disconnect errors: ${(discErr ? discErr.values.rate * 100 : 0).toFixed(2)}%\n`;
  out += `Connection success: ${(reconnect ? reconnect.values.rate * 100 : 0).toFixed(2)}%\n`;
  out += `Heartbeat missed: ${(hbMiss ? hbMiss.values.rate * 100 : 0).toFixed(2)}%\n`;
  out += `Broadcasts received: ${broadcast ? broadcast.values.count : 0}\n`;
  if (connDur) {
    out += `Connection duration: med=${connDur.values.med}ms, p95=${connDur.values['p(95)']}ms\n`;
  }

  return {
    stdout: out,
    'reports/websocket-stress-summary.json': JSON.stringify(data, null, 2),
  };
}
