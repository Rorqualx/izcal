// HGBA DRM spike: EME observer (initialization script, injected into every frame).
//
// READ-ONLY OBSERVATION. This script must not change page behaviour:
//  - Wrapped EME methods call the original with the same `this` and arguments and
//    return the original's return value (the same promise object, not a derived one).
//  - Event listeners are added passively (capture listeners on window, plus
//    listeners on MediaKeySession objects). They never call preventDefault or
//    stopPropagation.
//  - No DOM is modified, no globals are added, no requests are made.
//  - Known side effect (documented): attaching an observer to the promise returned
//    by requestMediaKeySystemAccess marks it as handled, so a page that ignores a
//    rejection will not see an `unhandledrejection` event for it.
//
// Records go to the host via wry's IPC bridge (window.ipc.postMessage); the host adds
// timestamps, OS, container and the sender origin, and appends a JSON line.
(() => {
  'use strict';
  const ipc = window.ipc;
  const send = (rec) => {
    try {
      const b = ipc || window.ipc;
      if (!b) return;
      rec.frameOrigin = location.origin;
      rec.isTop = window === window.top;
      rec.topOrigin = topOrigin();
      b.postMessage(JSON.stringify(rec));
    } catch (_) { /* never surface observer errors to the page */ }
  };
  function topOrigin() {
    try {
      if (window === window.top) return location.origin;
      const ao = location.ancestorOrigins;
      if (ao && ao.length) return ao[ao.length - 1];
      return window.top.location.origin;
    } catch (_) { return null; }
  }
  const summarizeConfigs = (configs) => {
    try {
      return Array.from(configs || []).map((c) => ({
        initDataTypes: c.initDataTypes || null,
        video: (c.videoCapabilities || []).map((v) => [v.contentType, v.robustness || '']),
        audio: (c.audioCapabilities || []).map((a) => [a.contentType, a.robustness || '']),
        persistentState: c.persistentState || null,
        distinctiveIdentifier: c.distinctiveIdentifier || null,
        sessionTypes: c.sessionTypes || null,
      }));
    } catch (_) { return null; }
  };
  // Replace a method with a pass-through wrapper, keeping name, length and descriptor flags.
  const wrap = (proto, name, makeWrapper) => {
    try {
      if (!proto) return;
      const d = Object.getOwnPropertyDescriptor(proto, name);
      if (!d || typeof d.value !== 'function') return;
      const orig = d.value;
      const w = makeWrapper(orig);
      Object.defineProperty(w, 'name', { value: orig.name, configurable: true });
      Object.defineProperty(w, 'length', { value: orig.length, configurable: true });
      Object.defineProperty(proto, name, { ...d, value: w });
    } catch (_) { /* leave the original in place */ }
  };
  const observe = (p, onOk, onErr) => {
    try { if (p && typeof p.then === 'function') p.then(onOk, onErr); } catch (_) {}
    return p;
  };

  const keySystemOf = new WeakMap(); // MediaKeys -> key system string

  // 1. Capability requests (a bare probe; per the plan this must NOT trigger a switch).
  wrap(window.Navigator && Navigator.prototype, 'requestMediaKeySystemAccess', (orig) => ({
    requestMediaKeySystemAccess(keySystem, configs) {
      const p = orig.apply(this, arguments);
      const ks = String(keySystem);
      send({ kind: 'mksa_request', keySystem: ks, configs: summarizeConfigs(configs) });
      return observe(p,
        (access) => {
          let granted = null;
          try { granted = summarizeConfigs([access.getConfiguration()])[0]; } catch (_) {}
          send({ kind: 'mksa_result', keySystem: ks, granted: true, config: granted });
        },
        (err) => send({ kind: 'mksa_result', keySystem: ks, granted: false, error: err && err.name }));
    },
  }).requestMediaKeySystemAccess);

  // 2. MediaKeys creation (part of the auto-trigger rule).
  wrap(window.MediaKeySystemAccess && MediaKeySystemAccess.prototype, 'createMediaKeys', (orig) => ({
    createMediaKeys() {
      const ks = (() => { try { return this.keySystem; } catch (_) { return null; } })();
      return observe(orig.apply(this, arguments),
        (mk) => { try { keySystemOf.set(mk, ks); } catch (_) {} send({ kind: 'mediakeys_created', keySystem: ks }); },
        (err) => send({ kind: 'mediakeys_failed', keySystem: ks, error: err && err.name }));
    },
  }).createMediaKeys);

  // 3. Sessions: observe license requests (`message`) and key status changes.
  wrap(window.MediaKeys && MediaKeys.prototype, 'createSession', (orig) => ({
    createSession() {
      const s = orig.apply(this, arguments);
      const ks = keySystemOf.get(this) || null;
      try {
        s.addEventListener('message', (ev) => send({
          kind: 'license_request', keySystem: ks, messageType: ev.messageType,
          bytes: ev.message ? ev.message.byteLength : null,
        }));
        s.addEventListener('keystatuseschange', () => {
          const st = [];
          try { s.keyStatuses.forEach((status) => st.push(status)); } catch (_) {}
          send({ kind: 'key_statuses', keySystem: ks, statuses: st });
        });
      } catch (_) {}
      return s;
    },
  }).createSession);

  wrap(window.HTMLMediaElement && HTMLMediaElement.prototype, 'setMediaKeys', (orig) => ({
    setMediaKeys(mk) {
      const ks = mk ? (keySystemOf.get(mk) || null) : null;
      return observe(orig.apply(this, arguments),
        () => send({ kind: 'mediakeys_attached', keySystem: ks, detached: !mk }),
        (err) => send({ kind: 'mediakeys_attach_failed', keySystem: ks, error: err && err.name }));
    },
  }).setMediaKeys);

  // Legacy WebKit (prefixed) EME, still used by some FairPlay players.
  wrap(window.HTMLMediaElement && HTMLMediaElement.prototype, 'webkitSetMediaKeys', (orig) => ({
    webkitSetMediaKeys(mk) {
      const r = orig.apply(this, arguments);
      let ks = null; try { ks = mk && mk.keySystem; } catch (_) {}
      send({ kind: 'mediakeys_created', keySystem: ks, legacy: true });
      return r;
    },
  }).webkitSetMediaKeys);
  wrap(window.WebKitMediaKeys && WebKitMediaKeys.prototype, 'createSession', (orig) => ({
    createSession() {
      const s = orig.apply(this, arguments);
      let ks = null; try { ks = this.keySystem; } catch (_) {}
      try {
        s.addEventListener('webkitkeymessage', (ev) => send({
          kind: 'license_request', keySystem: ks, legacy: true,
          bytes: ev.message ? ev.message.byteLength : null,
        }));
      } catch (_) {}
      return s;
    },
  }).createSession);

  // 4. `encrypted` events (capture phase on window sees non-bubbling media events).
  const onEncrypted = (ev) => {
    send({ kind: 'encrypted', initDataType: ev.initDataType || null, legacy: ev.type !== 'encrypted' });
  };
  try {
    window.addEventListener('encrypted', onEncrypted, { capture: true, passive: true });
    window.addEventListener('webkitneedkey', onEncrypted, { capture: true, passive: true });
  } catch (_) {}

  // 5. Resolution sampling: largest playing <video>, reported on change and every 15 s.
  let last = '';
  let lastSent = 0;
  setInterval(() => {
    try {
      let best = null;
      for (const v of document.querySelectorAll('video')) {
        if (v.paused || v.readyState < 2) continue;
        const px = v.videoWidth * v.videoHeight;
        if (!best || px > best.px) best = { v, px };
      }
      if (!best) return;
      const v = best.v;
      let q = null;
      try {
        const pq = v.getVideoPlaybackQuality();
        q = { total: pq.totalVideoFrames, dropped: pq.droppedVideoFrames };
      } catch (_) {}
      let ks = null; try { ks = v.mediaKeys ? (keySystemOf.get(v.mediaKeys) || 'unknown') : null; } catch (_) {}
      const key = v.videoWidth + 'x' + v.videoHeight + '|' + ks;
      const now = Date.now();
      if (key !== last || now - lastSent > 15000) {
        last = key; lastSent = now;
        send({
          kind: 'video', width: v.videoWidth, height: v.videoHeight,
          encrypted: !!v.mediaKeys, keySystem: ks, frames: q,
          displayed: [v.clientWidth, v.clientHeight],
        });
      }
    } catch (_) {}
  }, 2000);

  // 6. One page record per top-level document.
  if (window === window.top) {
    send({ kind: 'page', ua: navigator.userAgent, secureContext: window.isSecureContext,
           emeAvailable: typeof navigator.requestMediaKeySystemAccess === 'function' });
  }
})();
