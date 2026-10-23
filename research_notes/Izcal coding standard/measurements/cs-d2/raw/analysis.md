| benchmark | instr/iter off | instr/iter on | instr Δ% | instr spread (max-min)/med off, on | cycles Δ% | wall median off (µs) | wall median on (µs) | wall Δ% | wall IQR off / on (µs) |
|---|---|---|---|---|---|---|---|---|---|
| cssparser_tokenize_bootstrap | 12,715,332 | 13,633,035 | +7.22 | 0.0000%, 0.0000% | +3.90 | 3,207.67 | 3,369.45 | +5.04 | 6.43 / 12.82 |
| cssparser_tokenize_bulma | 33,525,890 | 36,155,031 | +7.84 | 0.0000%, 0.0001% | +7.18 | 7,788.52 | 8,336.77 | +7.04 | 26.19 / 280.54 |
| stylo_parse_bootstrap | 43,035,884 | 43,922,618 | +2.06 | 0.0000%, 0.0000% | -1.82 | 13,026.45 | 12,840.19 | -1.43 | 306.59 / 42.91 |
| stylo_parse_bulma | 110,081,265 | 113,202,265 | +2.84 | 0.0001%, 0.0001% | +0.56 | 28,151.98 | 28,483.29 | +1.18 | 405.06 / 283.99 |
| stylo_stylist_bootstrap | 5,370,018 | 5,685,994 | +5.88 | 0.0011%, 0.0006% | +0.73 | 1,647.09 | 1,663.87 | +1.02 | 4.07 / 9.18 |
| stylo_stylist_bulma | 8,534,934 | 8,944,996 | +4.80 | 0.0005%, 0.0007% | +0.16 | 2,646.87 | 2,654.98 | +0.31 | 4.54 / 8.38 |
| selectors_match_bootstrap_navbar | 4,073,682,938 | 4,054,777,604 | -0.46 | 0.0000%, 0.0000% | -0.30 | 834,418.01 | 835,936.29 | +0.18 | 582.15 / 1,171.27 |
| html5ever_treebuild_wikipedia | 65,369,681 | 70,488,309 | +7.83 | 0.0000%, 0.0000% | +5.59 | 14,617.35 | 15,517.69 | +6.16 | 1,310.67 / 166.60 |
| html5ever_treebuild_navbar | 92,191,237 | 98,634,332 | +6.99 | 0.0000%, 0.0000% | +9.24 | 19,607.82 | 21,044.08 | +7.32 | 514.87 / 97.07 |
| html5ever_tokenize_lipsum | 38,888 | 41,778 | +7.43 | 0.0304%, 0.0168% | -0.00 | 5.01 | 5.00 | -0.05 | 0.04 / 0.01 |
| html5ever_tokenize_lipsum_zh | 14,081 | 14,797 | +5.08 | 0.1888%, 0.1638% | +0.81 | 1.55 | 1.56 | +0.64 | 0.03 / 0.02 |
| html5ever_tokenize_medium_fragment | 423,715 | 465,974 | +9.97 | 0.0380%, 0.0053% | +9.42 | 79.53 | 88.47 | +11.24 | 1.53 / 5.06 |
| html5ever_tokenize_small_fragment | 41,292 | 45,170 | +9.39 | 0.1137%, 0.0532% | +8.11 | 7.18 | 7.92 | +10.34 | 0.32 / 0.03 |
| html5ever_tokenize_tiny_fragment | 7,533 | 7,724 | +2.54 | 0.0529%, 0.2620% | +1.95 | 0.74 | 0.82 | +11.47 | 0.00 / 0.00 |
| html5ever_tokenize_strong | 317,804 | 346,170 | +8.93 | 0.0156%, 0.0019% | +12.17 | 51.64 | 57.57 | +11.48 | 2.27 / 0.10 |
| url_join_wikipedia | 4,644,497 | 4,684,809 | +0.87 | 0.0003%, 0.0002% | +3.74 | 893.69 | 909.88 | +1.81 | 19.84 / 3.47 |
| url_parse_own_bench_strings | 27,152,027 | 27,678,024 | +1.94 | 0.0000%, 0.0000% | +1.76 | 5,514.39 | 5,629.93 | +2.10 | 60.41 / 54.86 |

| crate | n | instr geomean Δ% | cycles geomean Δ% | wall geomean Δ% |
|---|---|---|---|---|
| cssparser | 2 | +7.53 | +5.53 | +6.04 |
| stylo | 4 | +3.88 | -0.10 | +0.26 |
| selectors | 1 | -0.46 | -0.30 | +0.18 |
| html5ever | 8 | +7.25 | +5.83 | +7.23 |
| url | 2 | +1.40 | +2.75 | +1.95 |
| overall (geomean of crate geomeans) | 5 | +3.87 | +2.71 | +3.09 |
| overall (geomean of all benchmarks) | 17 | +5.32 | +3.64 | +4.37 |
| renderer-only (crate geomean, excl. url) | 4 | +4.50 | +2.70 | +3.38 |

- cssparser_tokenize_bootstrap: `rig check OK: revire kind=exclusive session=browser-61 | load 0.14/cpu (3.29 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:53:10-06:00` (exit 0)
- cssparser_tokenize_bulma: `rig check OK: revire kind=exclusive session=browser-61 | load 0.12/cpu (2.78 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:53:22-06:00` (exit 0)
- stylo_parse_bootstrap: `rig check OK: revire kind=exclusive session=browser-61 | load 0.10/cpu (2.39 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:53:36-06:00` (exit 0)
- stylo_parse_bulma: `rig check OK: revire kind=exclusive session=browser-61 | load 0.09/cpu (2.17 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:53:49-06:00` (exit 0)
- stylo_stylist_bootstrap: `rig check OK: revire kind=exclusive session=browser-61 | load 0.08/cpu (1.91 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:54:03-06:00` (exit 0)
- stylo_stylist_bulma: `rig check OK: revire kind=exclusive session=browser-61 | load 0.07/cpu (1.77 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:54:14-06:00` (exit 0)
- selectors_match_bootstrap_navbar: `rig check OK: revire kind=exclusive session=browser-61 | load 0.07/cpu (1.60 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:54:27-06:00` (exit 0)
- html5ever_treebuild_wikipedia: `rig check OK: revire kind=exclusive session=browser-61 | load 0.05/cpu (1.11 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:06-06:00` (exit 0)
- html5ever_treebuild_navbar: `rig check OK: revire kind=exclusive session=browser-61 | load 0.05/cpu (1.09 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:20-06:00` (exit 0)
- html5ever_tokenize_lipsum: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.92 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:36-06:00` (exit 0)
- html5ever_tokenize_lipsum_zh: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.93 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:48-06:00` (exit 0)
- html5ever_tokenize_medium_fragment: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.86 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:56:59-06:00` (exit 0)
- html5ever_tokenize_small_fragment: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.89 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:11-06:00` (exit 0)
- html5ever_tokenize_tiny_fragment: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.91 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:23-06:00` (exit 0)
- html5ever_tokenize_strong: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.93 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:36-06:00` (exit 0)
- url_join_wikipedia: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.94 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:46-06:00` (exit 0)
- url_parse_own_bench_strings: `rig check OK: revire kind=exclusive session=browser-61 | load 0.04/cpu (0.95 / 24 cpus) | bookings: b-4c0108 exclusive cpu by browser-61 until 16:42 | game lock: free q0 | 2026-10-18T13:57:58-06:00` (exit 0)
