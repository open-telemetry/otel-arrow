#!/usr/bin/env bash
# Drive OTAP pdata-optimization before/after experiments.
# For each image (baseline, optimized): retag to :latest, run all 8 OTAP-none
# passthrough suites at 400k+800k (20s), and save per-suite data to results/<img>/.
set -u
cd "$(dirname "$0")/.."   # tools/comparison_dashboard

SUITES=(
  dfe-metrics-otap-none-baseline
  dfe-metrics-otap-none-lowbatch
  dfe-metrics-otap-none-manyproc
  dfe-metrics-otap-none-compound
  dfe-logs-otap-none-baseline
  dfe-logs-otap-none-lowbatch
  dfe-logs-otap-none-manyproc
  dfe-logs-otap-none-compound
)
RATES="400k,800k"
RESULTS=.run-logs/results
mkdir -p "$RESULTS"

for IMG in baseline optimized; do
  echo "######## IMAGE: $IMG ########"
  docker tag df_engine:$IMG df_engine:latest
  for s in "${SUITES[@]}"; do
    echo "==== $IMG / $s ===="
    docker rm -f load-generator backend-service go-collector 2>/dev/null
    MAX_RUNTIME=600 .run-logs/run-with-watch.sh "suites/dfe/$s.yaml" \
      --observation-interval 20 --tests "$RATES" 2>&1 | tail -1
    slug=$(grep '^slug:' "suites/dfe/$s.yaml" | awk '{print $2}')
    mkdir -p "$RESULTS/$IMG"
    rm -rf "$RESULTS/$IMG/$slug"
    cp -r ".site/data/suite/$slug" "$RESULTS/$IMG/$slug"
  done
done
echo "ALL DONE"
