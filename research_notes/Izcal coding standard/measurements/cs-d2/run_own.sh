#!/bin/bash
# Crates' own benches (html5ever criterion, url bencher), interleaved off/on, pinned to CPU 4.
H="joe@192.168.50.9"
for round in 1 2 3; do
  if [ $((round % 2)) = 1 ]; then order="off on"; else order="on off"; fi
  for b in $order; do
    chk=$(rig check revire --kind exclusive --session browser-61 2>&1); rc=$?
    echo "== rig check (round $round $b): $chk exit=$rc"
    if [ $rc != 0 ]; then echo "rig check failed; stopping"; exit 1; fi
    echo "== html5ever round=$round build=$b"
    ssh $H "cd ~/izcal-measure/cs-d2/html5ever/html5ever && taskset -c 4 \$(ls ../../tgt-h5-$b/release/deps/html5ever-* | grep -v '\.d$') --bench --noplot --warm-up-time 2 --measurement-time 5 --discard-baseline 2>&1 | grep -E '^html|time:'"
    echo "== url round=$round build=$b"
    ssh $H "cd ~/izcal-measure/cs-d2/url/url && taskset -c 4 \$(ls ../../tgt-url-$b/release/deps/parse_url-* | grep -v '\.d$') --bench 2>&1 | grep 'ns/iter'"
  done
done
echo OWN DONE
