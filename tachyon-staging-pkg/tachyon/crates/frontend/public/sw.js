// Tachyon PWA Service Worker v3
const CACHE_VERSION = 'tachyon-v3';
const STATIC_CACHE = `${CACHE_VERSION}-static`;
const API_CACHE = `${CACHE_VERSION}-api`;
const OFFLINE_CACHE = `${CACHE_VERSION}-offline`;

const PRECACHE_URLS = [
  '/',
  '/index.html',
  '/manifest.json',
  '/offline.html',
];

const STATIC_EXTENSIONS = ['.wasm', '.js', '.css', '.svg', '.png', '.ico', '.woff', '.woff2', '.jpg', '.jpeg', '.gif', '.webp'];
const API_CACHE_TTL_MS = 30_000;
const API_CACHE_MAX_ENTRIES = 50;

// ---------------------------------------------------------------------------
// Install: pre-cache app shell
// ---------------------------------------------------------------------------
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(STATIC_CACHE).then((cache) => cache.addAll(PRECACHE_URLS))
  );
  self.skipWaiting();
});

// ---------------------------------------------------------------------------
// Activate: purge old caches
// ---------------------------------------------------------------------------
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys
          .filter((key) => key.startsWith('tachyon-') && key !== STATIC_CACHE && key !== API_CACHE && key !== OFFLINE_CACHE)
          .map((key) => {
            console.log('[SW] Purging old cache:', key);
            return caches.delete(key);
          })
      )
    )
  );
  self.clients.claim();
});

// ---------------------------------------------------------------------------
// Background Sync: replay pending writes when back online
// ---------------------------------------------------------------------------
self.addEventListener('sync', (event) => {
  if (event.tag === 'tachyon-sync-pending') {
    event.waitUntil(syncPendingChanges());
  }
});

async function syncPendingChanges() {
  const clients = await self.clients.matchAll();
  for (const client of clients) {
    client.postMessage({ type: 'SYNC_START' });
  }

  try {
    const db = await openDB();
    const tx = db.transaction('pending_changes', 'readonly');
    const store = tx.objectStore('pending_changes');
    const changes = await idbRequest(store.getAll());

    if (!changes || changes.length === 0) return;

    for (const change of changes) {
      try {
        const response = await fetch(`/api/v1/documents/${change.document_id}/changes`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            operation: change.operation,
            payload: change.payload,
          }),
        });

        if (response.ok) {
          const deleteTx = db.transaction('pending_changes', 'readwrite');
          const deleteStore = deleteTx.objectStore('pending_changes');
          await idbRequest(deleteStore.delete(change.id));
        }
      } catch (err) {
        console.error('[SW] Failed to sync change:', change.id, err);
      }
    }

    for (const client of clients) {
      client.postMessage({ type: 'SYNC_COMPLETE', synced: changes.length });
    }
  } catch (err) {
    console.error('[SW] syncPendingChanges failed:', err);
  }
}

function openDB() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('tachyon_offline', 1);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

function idbRequest(request) {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

// ---------------------------------------------------------------------------
// Message handler: trigger sync from client
// ---------------------------------------------------------------------------
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'TRIGGER_SYNC') {
    self.registration.sync.register('tachyon-sync-pending').catch((err) => {
      console.warn('[SW] Background sync registration failed:', err);
    });
  }
});

// ---------------------------------------------------------------------------
// Fetch: routing strategy per resource type
// ---------------------------------------------------------------------------
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  if (request.method !== 'GET' && request.method !== 'POST') {
    return;
  }

  // POST requests: queue for background sync if offline
  if (request.method === 'POST' && url.pathname.startsWith('/api/')) {
    event.respondWith(
      fetch(request.clone()).catch(async () => {
        const body = await request.clone().text().catch(() => null);
        if (body) {
          await queueOfflineRequest(url.pathname, body);
          self.registration.sync.register('tachyon-sync-pending').catch(() => {});
        }
        return new Response(JSON.stringify({ queued: true }), {
          status: 202,
          headers: { 'Content-Type': 'application/json' },
        });
      })
    );
    return;
  }

  if (url.origin !== self.location.origin && !url.pathname.startsWith('/api/')) {
    return;
  }

  // 1. API requests: network-first, short cache TTL
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(networkFirstWithCache(request, API_CACHE_TTL_MS));
    return;
  }

  // 2. Static assets: cache-first for instant offline load
  if (STATIC_EXTENSIONS.some((ext) => url.pathname.endsWith(ext))) {
    event.respondWith(cacheFirst(request));
    return;
  }

  // 3. Navigation (HTML pages): network-first, offline fallback
  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const clone = response.clone();
          caches.open(STATIC_CACHE).then((cache) => cache.put(request, clone));
          return response;
        })
        .catch(() =>
          caches.match(request).then((cached) => cached || caches.match('/offline.html'))
        )
    );
    return;
  }

  // 4. Everything else: network-first with cache fallback
  event.respondWith(networkFirstWithCache(request, 60_000));
});

async function queueOfflineRequest(path, body) {
  try {
    const db = await openDB();
    const tx = db.transaction('pending_changes', 'readwrite');
    const store = tx.objectStore('pending_changes');
    const id = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    await idbRequest(store.put({
      id,
      document_id: path.split('/').filter(Boolean).pop() || 'unknown',
      operation: 'api_post',
      payload: body,
      created_at: new Date().toISOString(),
      retry_count: 0,
    }));
  } catch (err) {
    console.error('[SW] queueOfflineRequest failed:', err);
  }
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

async function cacheFirst(request) {
  const cached = await caches.match(request);
  if (cached) return cached;

  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(STATIC_CACHE);
      cache.put(request, response.clone());
    }
    return response;
  } catch {
    return new Response('Offline', { status: 503 });
  }
}

async function networkFirstWithCache(request, ttlMs) {
  const cached = await caches.match(request);
  const now = Date.now();
  const cacheTimestamp = cached && cached.headers.get('sw-cache-timestamp');
  const stale = cacheTimestamp && (now - parseInt(cacheTimestamp, 10)) > ttlMs;

  if (cached && !stale) {
    fetchAndCache(request, API_CACHE).catch(() => {});
    return cached;
  }

  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(API_CACHE);
      const headers = new Headers(response.headers);
      headers.set('sw-cache-timestamp', now.toString());
      const body = await response.clone().blob();
      const cachedResponse = new Response(body, { headers, status: response.status });
      const entries = await cache.keys();
      if (entries.length >= API_CACHE_MAX_ENTRIES) {
        await cache.delete(entries[0]);
      }
      await cache.put(request, cachedResponse);
    }
    return response;
  } catch {
    return cached || new Response(JSON.stringify({ error: 'offline' }), {
      status: 503,
      headers: { 'Content-Type': 'application/json' },
    });
  }
}

async function fetchAndCache(request, cacheName) {
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(cacheName);
      const headers = new Headers(response.headers);
      headers.set('sw-cache-timestamp', Date.now().toString());
      const body = await response.clone().blob();
      await cache.put(request, new Response(body, { headers, status: response.status }));
    }
  } catch {}
}
