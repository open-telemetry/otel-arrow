#!/usr/bin/env bash
# Repeat 1000k/120s standard + many-proc (metrics, logs), before/after, 3x.
set -u
cd "$(dirname "$0")/.."
SUITES=(
  dfe-metrics-otap-none-baseline
  dfe-logs-otap-none-baseline
  dfe-metrics-otap-none-manyproc
  dfe-logs-otap-none-manyproc
)
RESULTS=.run-logs/results-1000k-rep
for RUN in 1 2 3; do
  for IMG in baseline optimized; do
    echo "######## RUN $RUN / IMAGE $IMG ########"
    docker tag df_engine:$IMG df_engine:latest
    for s in "${SUITES[@]}"; do
      docker rm -f load-generator backend-service go-collector 2>/dev/null
      MAX_RUNTIME=900 .run-logs/run-with-watch.sh "suites/dfe/$s.yaml" \
        --observation-interval 120 --tests 1000k >/dev/null 2>&1
      rc=$?
      slug=$(grep '^slug:' "suites/dfe/$s.yaml" | awk '{print $2}')
      dest="$RESULTS/run$RUN/$IMG/$slug"
      mkdir -p "$(dirname "$dest")"; rm -rf "$dest"
      cp -r ".site/data/suite/$slug" "$dest"
      echo "==== DONE run$RUN $IMG / $s rc=$rc ===="
    done
  done
done
echo "REPEAT DONE"
