// Tachyon PWA Service Worker v2
// Cache versioning: bump CACHE_VERSION to force cache refresh across all clients.
const CACHE_VERSION = 'tachyon-v2';

// Pre-cache shell resources for instant offline load.
const PRECACHE_URLS = [
  '/',
  '/index.html',
  '/manifest.json',
  '/offline.html',
];

// Stale-while-revalidate for versioned static assets (WASM, JS, CSS, images).
const STATIC_EXTENSIONS = ['.wasm', '.js', '.css', '.svg', '.png', '.ico', '.woff', '.woff2'];

// Network-first for API calls with offline fallback to cached 401/error page.
const API_CACHE_TTL_MS = 30_000; // 30 seconds

// ---------------------------------------------------------------------------
// Install: pre-cache app shell
// ---------------------------------------------------------------------------
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_VERSION).then((cache) => cache.addAll(PRECACHE_URLS))
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
          .filter((key) => key !== CACHE_VERSION && key.startsWith('tachyon-'))
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
// Fetch: routing strategy per resource type
// ---------------------------------------------------------------------------
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }

  // Skip cross-origin requests (except our own API)
  if (url.origin !== self.location.origin && !url.pathname.startsWith('/api/')) {
    return;
  }

  // 1. API requests: network-first, short cache TTL
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(networkFirstWithCache(request, API_CACHE_TTL_MS));
    return;
  }

  // 2. Static assets: stale-while-revalidate
  if (STATIC_EXTENSIONS.some((ext) => url.pathname.endsWith(ext))) {
    event.respondWith(staleWhileRevalidate(request));
    return;
  }

  // 3. Navigation (HTML pages): network-first, offline fallback
  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request)
        .catch(() => caches.match('/offline.html').then((r) => r || caches.match('/index.html')))
    );
    return;
  }

  // 4. Everything else: network-first with cache fallback
  event.respondWith(networkFirstWithCache(request, 60_000));
});

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

function networkFirstWithCache(request, ttlMs) {
  return caches.match(request).then((cached) => {
    const now = Date.now();
    const maxAge = cached && cached.headers.get('sw-cache-timestamp');
    const stale = maxAge && (now - parseInt(maxAge, 10)) > ttlMs;

    if (cached && !stale) {
      return cached;
    }

    return fetch(request)
      .then((response) => {
        if (response.ok) {
          const clone = response.clone();
          const headers = new Headers(clone.headers);
          headers.set('sw-cache-timestamp', now.toString());
          const bodyPromise = clone.blob();
          bodyPromise.then((body) => {
            const cachedResponse = new Response(body, { headers, status: clone.status });
            caches.open(CACHE_VERSION).then((cache) => cache.put(request, cachedResponse));
          });
        }
        return response;
      })
      .catch(() => cached || new Response('Offline', { status: 503 }));
  });
}

function staleWhileRevalidate(request) {
  return caches.match(request).then((cached) => {
    const fetchPromise = fetch(request).then((response) => {
      if (response.ok) {
        const clone = response.clone();
        caches.open(CACHE_VERSION).then((cache) => cache.put(request, clone));
      }
      return response;
    }).catch(() => cached);

    return cached || fetchPromise;
  });
}
