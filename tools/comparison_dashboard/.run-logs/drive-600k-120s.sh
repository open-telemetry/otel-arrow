#!/usr/bin/env bash
# Re-run the significant OTAP pdata-opt configs at 600k with a 120s window,
# before/after. Skips compound. Results saved separately from the 400k/800k run.
set -u
cd "$(dirname "$0")/.."
SUITES=(
  dfe-metrics-otap-none-baseline
  dfe-logs-otap-none-baseline
  dfe-metrics-otap-none-lowbatch
  dfe-logs-otap-none-lowbatch
  dfe-metrics-otap-none-manyproc
  dfe-logs-otap-none-manyproc
)
RESULTS=.run-logs/results-600k-120s
mkdir -p "$RESULTS"
for IMG in baseline optimized; do
  echo "######## IMAGE: $IMG ########"
  docker tag df_engine:$IMG df_engine:latest
  for s in "${SUITES[@]}"; do
    echo "==== $IMG / $s ===="
    docker rm -f load-generator backend-service go-collector 2>/dev/null
    MAX_RUNTIME=900 .run-logs/run-with-watch.sh "suites/dfe/$s.yaml" \
      --observation-interval 120 --tests 600k 2>&1 | tail -1
    slug=$(grep '^slug:' "suites/dfe/$s.yaml" | awk '{print $2}')
    mkdir -p "$RESULTS/$IMG"; rm -rf "$RESULTS/$IMG/$slug"
    cp -r ".site/data/suite/$slug" "$RESULTS/$IMG/$slug"
  done
done
echo "600K-120S DONE"
