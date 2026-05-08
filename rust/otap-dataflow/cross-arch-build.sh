set -exu

if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then
  RUST_TARGET=x86_64-unknown-linux-gnu
  if [ "${TARGETPLATFORM}" != "${BUILDPLATFORM}" ]; then
    apt-get update && apt-get install -y gcc-x86-64-linux-gnu g++-x86-64-linux-gnu
    export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
    export CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++
    export AR_x86_64_unknown_linux_gnu=x86_64-linux-gnu-ar
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
  fi
elif [ "${TARGETPLATFORM}" = "linux/arm64" ]; then
  RUST_TARGET=aarch64-unknown-linux-gnu
  if [ "${TARGETPLATFORM}" != "${BUILDPLATFORM}" ]; then
    apt-get update && apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
    export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
  fi
else
  echo "Unsupported target platform: ${TARGETPLATFORM}"
  exit 1
fi

rustup target add "${RUST_TARGET}"
cargo build --release --features "$FEATURES" --target="${RUST_TARGET}"
cp "target/${RUST_TARGET}/release/df_engine" .
