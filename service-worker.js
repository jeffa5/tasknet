var cacheName = "tasknet-0";
var contentToCache = [
  "index.html",
  "pkg/package.js",
];

self.addEventListener('install', (e) => {
  console.log('[Service Worker] Install');
  e.waitUntil(
    caches.open(cacheName).then((cache) => {
          console.log('[Service Worker] Caching all: app shell and content');
      return cache.addAll(contentToCache);
    })
  );
});

function promiseAny(promises) {
  return new Promise((resolve, reject) => {
    // make sure promises are all promises
    promises = promises.map((p) => Promise.resolve(p));
    // resolve this promise as soon as one resolves
    promises.forEach((p) => p.then(resolve));
    // reject if all promises reject
    promises.reduce((a, b) => a.catch(() => b)).catch(() => reject(Error('All failed')));
  });
}


self.addEventListener('fetch', (event) => {
  event.respondWith(promiseAny([
    caches.match(event.request).then((r) => {
      console.log("[Service Worker] Cached resource: " + event.request.url);
      return r;
    }),
    fetch(event.request).then((response) => {
      return caches.open(cacheName).then((cache) => {
        console.log("[Service Worker] Retrieved resource: "+event.request.url);
        cache.put(event.request, response.clone());
        return response;
      })
    })
  ]));
});
