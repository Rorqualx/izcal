#!/bin/bash
# Fetch each feed file once into feeds/<UTC timestamp>/. Only feed hosts are contacted;
# no listed URL is ever requested or resolved. Feed contents are untrusted data.
set -u
FEEDS_ROOT="$1"
TS=$(date -u +%Y%m%dT%H%M%SZ)
OUT="$FEEDS_ROOT/$TS"
mkdir -p "$OUT"
UA="${IZCAL_MEASURE_UA:-izcal-sec-d7-measurement/0.1 (research)}"

fetch() {
  local name="$1" url="$2"; shift 2
  local start end code
  start=$(date -u +%s)
  code=$(curl -sS -L --proto '=https' --max-time 300 --max-filesize 400000000 \
    -A "$UA" -D "$OUT/$name.headers" -o "$OUT/$name.body" -w '%{http_code} %{size_download} %{url_effective}' "$@" "$url" 2>"$OUT/$name.err")
  end=$(date -u +%s)
  echo "$name	$url	$code	$((end-start))s	$(date -u +%FT%TZ)" >> "$OUT/_index.tsv"
}

fetch openphish        "https://openphish.com/feed.txt"
fetch openphish_gh     "https://raw.githubusercontent.com/openphish/public_feed/refs/heads/main/feed.txt"
fetch urlhaus_online   "https://urlhaus.abuse.ch/downloads/text_online/"
fetch urlhaus_recent   "https://urlhaus.abuse.ch/downloads/csv_recent/"
fetch threatfox_recent "https://threatfox.abuse.ch/export/csv/recent/"
fetch bazaar_sha256    "https://bazaar.abuse.ch/export/txt/sha256/recent/"
fetch pdb_links_new    "https://raw.githubusercontent.com/Phishing-Database/Phishing.Database/master/phishing-links-NEW-today.txt"
fetch pdb_domains_new  "https://raw.githubusercontent.com/Phishing-Database/Phishing.Database/master/phishing-domains-NEW-today.txt"
fetch pdb_domains_act  "https://raw.githubusercontent.com/Phishing-Database/Phishing.Database/master/phishing-domains-ACTIVE.txt"
fetch phisharmy        "https://phishing.army/download/phishing_army_blocklist.txt"
fetch phisharmy_ext    "https://phishing.army/download/phishing_army_blocklist_extended.txt"
# phishstats.info/phish_score.csv returned 404 on 2026-10-18; the CSV dump is gone. Anonymous API: 50 req/day/IP.
fetch phishstats_api   "https://api.phishstats.info/api/phishing?_sort=-id&_size=500"
fetch certpl           "https://hole.cert.pl/domains/v2/domains.txt"
fetch phishunt         "https://phishunt.io/feed.txt"
fetch phishtank        "https://data.phishtank.com/data/online-valid.csv"
echo "$OUT"
