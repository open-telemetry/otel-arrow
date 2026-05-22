#!/usr/bin/env bash
set -u
cd "$(dirname "$0")/.."
SUITES=(dfe-metrics-otap-none-compound dfe-logs-otap-none-compound)
RESULTS=.run-logs/results
for IMG in baseline optimized; do
  echo "######## IMAGE: $IMG ########"
  docker tag df_engine:$IMG df_engine:latest
  for s in "${SUITES[@]}"; do
    echo "==== $IMG / $s ===="
    docker rm -f load-generator backend-service go-collector 2>/dev/null
    MAX_RUNTIME=600 .run-logs/run-with-watch.sh "suites/dfe/$s.yaml" \
      --observation-interval 20 --tests 400k,800k 2>&1 | tail -1
    slug=$(grep '^slug:' "suites/dfe/$s.yaml" | awk '{print $2}')
    mkdir -p "$RESULTS/$IMG"; rm -rf "$RESULTS/$IMG/$slug"
    cp -r ".site/data/suite/$slug" "$RESULTS/$IMG/$slug"
  done
done
echo "COMPOUND DONE"
