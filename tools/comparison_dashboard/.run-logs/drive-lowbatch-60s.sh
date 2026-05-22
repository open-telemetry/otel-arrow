#!/usr/bin/env bash
# Low-batch and compound (low-batch+many-proc), metrics+logs, before/after at 60s.
# Rate is the first arg (default 400k). Warm-up run first to avoid cold-start skew.
set -u
cd "$(dirname "$0")/.."
RATE="${1:-400k}"
SUITES=(
  dfe-metrics-otap-none-lowbatch
  dfe-logs-otap-none-lowbatch
  dfe-metrics-otap-none-compound
  dfe-logs-otap-none-compound
)
RESULTS=.run-logs/results-lowbatch-60s-$RATE
mkdir -p "$RESULTS"

echo "==== WARMUP ===="
docker tag df_engine:baseline df_engine:latest
docker rm -f load-generator backend-service go-collector 2>/dev/null
MAX_RUNTIME=600 .run-logs/run-with-watch.sh suites/dfe/dfe-metrics-otap-none-baseline.yaml \
  --observation-interval 20 --tests 100k >/dev/null 2>&1
echo "==== WARMUP DONE ===="

for IMG in baseline optimized; do
  echo "######## IMAGE $IMG ($RATE) ########"
  docker tag df_engine:$IMG df_engine:latest
  for s in "${SUITES[@]}"; do
    docker rm -f load-generator backend-service go-collector 2>/dev/null
    MAX_RUNTIME=900 .run-logs/run-with-watch.sh "suites/dfe/$s.yaml" \
      --observation-interval 60 --tests "$RATE" >/dev/null 2>&1
    rc=$?
    slug=$(grep '^slug:' "suites/dfe/$s.yaml" | awk '{print $2}')
    dest="$RESULTS/$IMG/$slug"
    mkdir -p "$(dirname "$dest")"; rm -rf "$dest"
    cp -r ".site/data/suite/$slug" "$dest"
    echo "==== DONE $IMG / $s rc=$rc ===="
  done
done
echo "LOWBATCH-60S-$RATE DONE"
