// HGBA M0 determinism probe (design doc Section 3.3). Every observable goes
// into one of three channels: R (Math.random), D (Date.now), G (GC-observable:
// WeakRef.deref and FinalizationRegistry callbacks). The embedder calls
// step(i) repeatedly; between some steps it forces a full GC, pumps the
// platform's foreground tasks (FinalizationRegistry cleanup) and runs a
// microtask checkpoint (which clears WeakRef keep-alive sets).
var LOG = { R: [], D: [], G: [] };
var finalized = [];
var registry = new FinalizationRegistry((token) => { finalized.push(token); });
var weakRefs = [];
var keep = [];
var churn = [];

function makeNode(i, j) {
  return { id: i * 1000 + j, payload: new Array(16).fill(i ^ j), next: null, s: "n" + i + "_" + j };
}

function step(i) {
  LOG.R.push(Math.random().toFixed(17));
  LOG.D.push(Date.now());
  // Allocate a linked batch; register every node, keep a WeakRef to some,
  // keep a few strongly for a while.
  var head = null;
  for (var j = 0; j < 200; j++) {
    var n = makeNode(i, j);
    n.next = head; head = n;
    registry.register(n, n.id);
    if (j % 25 === 0) weakRefs.push([n.id, new WeakRef(n)]);
  }
  if (i % 3 === 0) keep.push(head);
  if (keep.length > 4) keep.shift();
  // Allocation churn so V8 runs allocation-triggered (non-forced) GCs.
  var a = [];
  for (var k = 0; k < 3000; k++) a.push({ k: k, v: [k, i], t: "x" + k });
  churn.push(a);
  if (churn.length > 3) churn.shift();
  // Some JIT-able work.
  var acc = 0;
  for (var k = 0; k < 20000; k++) acc = (acc + ((k * 2654435761) >>> 0) % 977) | 0;
  // GC observables, sampled every step.
  var alive = 0, dead = [];
  for (var w of weakRefs) { if (w[1].deref() === undefined) dead.push(w[0]); else alive++; }
  LOG.G.push("s" + i + ":alive=" + alive + ":dead=" + dead.length + ":fin=" + finalized.length +
             ":last=" + finalized.slice(-3).join("/") + ":acc=" + acc);
}

function finish() {
  LOG.G.push("final:fin=" + finalized.length + ":order=" + finalized.join(","));
  return JSON.stringify(LOG);
}
