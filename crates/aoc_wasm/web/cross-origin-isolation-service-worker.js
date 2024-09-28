// This is a hack to set Cross-Origin-Embedder-Policy/Cross-Origin-Opener-Policy (required to send SharedArrayBuffer
// instances across workers/use multithreaded WebAssembly) on GitHub pages

if (typeof window === "undefined") {
    self.addEventListener("install", () => self.skipWaiting());
    self.addEventListener("activate", event => event.waitUntil(self.clients.claim()));

    self.addEventListener("fetch", event => {
        event.respondWith(fetch(event.request).then(response =>  {
            if (response.status === 0) return response;

            const headers = new Headers(response.headers);
            headers.set('Cross-Origin-Embedder-Policy', 'require-corp');
            headers.set('Cross-Origin-Opener-Policy', 'same-origin');

            return new Response(response.body, {status: response.status, statusText: response.statusText, headers});
        }));
    });
} else if (window.crossOriginIsolated && !navigator.serviceWorker?.controller) {
    console.log("cross-origin isolated without service worker"); // No service worker workaround needed
} else {
    if (window.crossOriginIsolated) {
        console.log("cross-origin isolated using service worker"); // Still re-register
    } else {
        console.log("not cross-origin isolated, trying service worker");
    }

    navigator.serviceWorker.register(document.currentScript.src).then(registration => {
        console.log(`service worker registered with scope ${registration.scope}`);

        registration.addEventListener("updatefound", () => {
            console.log("reloading page due to service worker update");
            window.location.reload();
        });
    }, error => {
        console.error("error registering service worker", error);
    });
}
