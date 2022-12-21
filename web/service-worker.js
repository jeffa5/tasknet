const version = 0;
const appName = "tasknet";
const cacheName = `${appName}-${version}`;
const contentToCache = [
  "index.html",
];

self.addEventListener('install', (event) => {
  console.log('[Service Worker]: Install');
  event.waitUntil(
    caches.open(cacheName).then((cache) => {
      console.log('[Service Worker]: Caching resources for offline');
      return cache.addAll(contentToCache);
    }).then(() => {
      console.log("[Service Worker]: Install complete")
    })
  );
});

function fetchedFromNetwork(event, response) {
  var cacheCopy = response.clone();

  console.log('[Service Worker]: Fetch response from network.', event.request.url);

  caches
    .open(cacheName)
    .then((cache) => {
      cache.put(event.request, cacheCopy);
    })
    .then(() => {
      console.log('[Service Worker]: Fetch response stored in cache.', event.request.url);
    });

  return response;
}

function unableToResolve () {
  console.log('[Service Worker]: Fetch request failed in both cache and network.');

  return new Response('<h1>Service Unavailable</h1>', {
    status: 503,
    statusText: 'Service Unavailable',
    headers: new Headers({
      'Content-Type': 'text/html'
    })
  });
}

self.addEventListener('fetch', (event) => {
  if (event.request.url.match("^\/auth\/.*$")) {
    return false;
  }
  event.respondWith(
    caches.match(event.request).then((cached) => {
      var networked = fetch(event.request).then((response) => fetchedFromNetwork(event, response), unableToResolve).catch(unableToResolve);
      console.log("[Service Worker]: Fetch event", cached ? "(cached)" : "(network)", event.request.url);
      return cached || networked
    })
  )
});

self.addEventListener("activate", function(event) {
  console.log('[Service Worker]: Activate event in progress.');

  event.waitUntil(
    caches
      .keys()
      .then((keys) => {
        return Promise.all(
          keys
            .filter((key) => {
              return key.startsWith(appName) && !key.startsWith(cacheName);
            })
            .map((key) => {
              console.log("[Service Worker]: Deleting cache key", key)
              return caches.delete(key);
            })
        );
      })
      .then(() => {
        console.log('[Service Worker]: Activate completed.');
      })
  );
});
