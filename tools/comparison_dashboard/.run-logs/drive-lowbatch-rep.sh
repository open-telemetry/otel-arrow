#!/usr/bin/env bash
# 3x repeat of low-batch (metrics, logs), before/after, 400k/60s. Warm-up first.
set -u
cd "$(dirname "$0")/.."
SUITES=(dfe-metrics-otap-none-lowbatch dfe-logs-otap-none-lowbatch)
RESULTS=.run-logs/results-lowbatch-rep

echo "==== WARMUP ===="
docker tag df_engine:baseline df_engine:latest
docker rm -f load-generator backend-service go-collector 2>/dev/null
MAX_RUNTIME=600 .run-logs/run-with-watch.sh suites/dfe/dfe-metrics-otap-none-baseline.yaml \
  --observation-interval 20 --tests 100k >/dev/null 2>&1
echo "==== WARMUP DONE ===="

for RUN in 1 2 3; do
  for IMG in baseline optimized; do
    echo "######## RUN $RUN / IMAGE $IMG ########"
    docker tag df_engine:$IMG df_engine:latest
    for s in "${SUITES[@]}"; do
      docker rm -f load-generator backend-service go-collector 2>/dev/null
      MAX_RUNTIME=900 .run-logs/run-with-watch.sh "suites/dfe/$s.yaml" \
        --observation-interval 60 --tests 400k >/dev/null 2>&1
      rc=$?
      slug=$(grep '^slug:' "suites/dfe/$s.yaml" | awk '{print $2}')
      dest="$RESULTS/run$RUN/$IMG/$slug"
      mkdir -p "$(dirname "$dest")"; rm -rf "$dest"
      cp -r ".site/data/suite/$slug" "$dest"
      echo "==== DONE run$RUN $IMG / $s rc=$rc ===="
    done
  done
done
echo "LOWBATCH-REP DONE"
