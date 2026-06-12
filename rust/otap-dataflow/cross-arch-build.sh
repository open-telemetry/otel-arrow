set -exu

if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then
  RUST_TARGET=x86_64-unknown-linux-gnu
  if [ "${TARGETPLATFORM}" != "${BUILDPLATFORM}" ]; then
    apt-get update && apt-get install -y gcc-x86-64-linux-gnu
  fi
elif [ "${TARGETPLATFORM}" = "linux/arm64" ]; then
  RUST_TARGET=aarch64-unknown-linux-gnu
  if [ "${TARGETPLATFORM}" != "${BUILDPLATFORM}" ]; then
    apt-get update && apt-get install -y gcc-aarch64-linux-gnu
  fi
else
  echo "Unsupported target platform: ${TARGETPLATFORM}"
  exit 1
fi

rustup target add "${RUST_TARGET}"

# rdkafka (used by kafka-receiver/kafka-exporter) requires libclang for
# bindgen at build time.
if echo "${FEATURES:-}" | grep -q 'kafka'; then
  apt-get update && apt-get install -y libclang-dev
fi

cargo build --release --features "$FEATURES" --target="${RUST_TARGET}"
cp "target/${RUST_TARGET}/release/df_engine" .
