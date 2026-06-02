#!/usr/bin/env bash
# Transform-correctness validation for the Fluent Bit `modify` filter.
#
# Pipeline:  DFE loadgen --(OTLP/gRPC)--> Fluent Bit (modify) --(OTLP/gRPC)--> DFE console backend
#
# Proves end to end that:
#   - attr-insert: `processing.engine=benchmark` appears on records that lacked it
#   - attr-rename: `exception.kind` appears and `exception.type` is gone
#
# Requires Docker access and the df_engine:latest image (the comparison-dashboard
# image) plus the Fluent Bit image. Run from this directory:
#   bash run.sh
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NET=fbval-net
DFE_IMAGE="${DFE_IMAGE:-df_engine:latest}"
FB_IMAGE="${FB_IMAGE:-fluent/fluent-bit:5.0}"
OBSERVE_SECS="${OBSERVE_SECS:-8}"

cleanup() {
  docker rm -f fbval-backend fbval-fluent-bit fbval-loadgen >/dev/null 2>&1
  docker network rm "$NET" >/dev/null 2>&1
}
trap cleanup EXIT

run_case() {
  local label="$1" fb_cfg="$2"
  echo "============================================================"
  echo "CASE: $label   (fb config: $fb_cfg)"
  echo "============================================================"
  cleanup
  docker network create "$NET" >/dev/null

  # 1) DFE console backend (OTLP/gRPC :1235 -> console)
  docker run -d --name fbval-backend --network "$NET" \
    -v "$HERE/inspect-backend.yaml:/home/nonroot/config.yaml:ro" \
    "$DFE_IMAGE" --config ./config.yaml --http-admin-bind 0.0.0.0:8080 >/dev/null

  # 2) Fluent Bit with the modify filter (OTLP/gRPC in :4317, out -> backend:1235)
  docker run -d --name fbval-fluent-bit --network "$NET" \
    -v "$HERE/$fb_cfg:/fluent-bit/etc/fluent-bit.yaml:ro" \
    "$FB_IMAGE" --config /fluent-bit/etc/fluent-bit.yaml >/dev/null

  sleep 3  # let FB + backend bind their listeners

  # 3) DFE loadgen -> Fluent Bit :4317
  docker run -d --name fbval-loadgen --network "$NET" \
    -v "$HERE/loadgen.yaml:/home/nonroot/config.yaml:ro" \
    "$DFE_IMAGE" --config ./config.yaml --http-admin-bind 0.0.0.0:8080 >/dev/null

  echo "observing for ${OBSERVE_SECS}s ..."
  sleep "$OBSERVE_SECS"

  echo "----- fluent-bit logs (tail) -----"
  docker logs --tail 25 fbval-fluent-bit 2>&1
  echo "----- backend (console exporter) received-record sample -----"
  docker logs fbval-backend 2>&1 | grep -iE "processing.engine|exception.kind|exception.type|end_user.id|user.id" | head -20
  echo "(if the section above is empty, no records reached the backend -- check FB logs)"
  echo
}

run_case "attr-insert" "fb-attr-insert.yaml"
run_case "attr-rename" "fb-attr-rename.yaml"

echo "Done. Manually confirm:"
echo "  insert -> 'processing.engine' / 'benchmark' present on records"
echo "  rename -> 'exception.kind' present AND 'exception.type' absent"
