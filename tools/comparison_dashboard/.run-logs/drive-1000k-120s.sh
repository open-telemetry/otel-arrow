#!/usr/bin/env bash
# Repeat standard + many-proc (metrics, logs) at 1000k/120s, before/after.
set -u
cd "$(dirname "$0")/.."
SUITES=(
  dfe-metrics-otap-none-baseline
  dfe-logs-otap-none-baseline
  dfe-metrics-otap-none-manyproc
  dfe-logs-otap-none-manyproc
)
RESULTS=.run-logs/results-1000k-120s
mkdir -p "$RESULTS"
for IMG in baseline optimized; do
  echo "######## IMAGE: $IMG ########"
  docker tag df_engine:$IMG df_engine:latest
  for s in "${SUITES[@]}"; do
    echo "==== START $IMG / $s ===="
    docker rm -f load-generator backend-service go-collector 2>/dev/null
    MAX_RUNTIME=900 .run-logs/run-with-watch.sh "suites/dfe/$s.yaml" \
      --observation-interval 120 --tests 1000k >/dev/null 2>&1
    rc=$?
    slug=$(grep '^slug:' "suites/dfe/$s.yaml" | awk '{print $2}')
    mkdir -p "$RESULTS/$IMG"; rm -rf "$RESULTS/$IMG/$slug"
    cp -r ".site/data/suite/$slug" "$RESULTS/$IMG/$slug"
    echo "==== DONE  $IMG / $s rc=$rc ===="
  done
done
echo "1000K-120S DONE"
