set -exu

if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then
  RUST_TARGET=x86_64-unknown-linux-musl
  apt-get update && apt-get install -y musl-tools
  export CC_x86_64_unknown_linux_musl=musl-gcc
  if [ "${TARGETPLATFORM}" != "${BUILDPLATFORM}" ]; then
    apt-get update && apt-get install -y gcc-x86-64-linux-gnu
  fi
elif [ "${TARGETPLATFORM}" = "linux/arm64" ]; then
  RUST_TARGET=aarch64-unknown-linux-musl
  # Always use the musl.cc cross-toolchain for arm64, regardless of whether this
  # is a native or cross build. The Debian gcc-aarch64-linux-gnu package (and
  # musl-gcc wrappers) use glibc system headers; glibc >= 2.38 maps _GNU_SOURCE
  # to _ISOC23_SOURCE, which redirects strtol/sscanf to __isoc23_strtol/
  # __isoc23_sscanf. Those C23 symbols don't exist in musl libc, causing link
  # failures in C dependencies that define _GNU_SOURCE (e.g. aws-lc-sys/bcm.c).
  # The musl.cc toolchain ships its own musl headers, so this redirection never
  # occurs, fixing the issue cleanly for both native and cross builds.
  MUSL_CROSS_TARBALL=aarch64-linux-musl-cross.tgz
  MUSL_CROSS_SHA256=c909817856d6ceda86aa510894fa3527eac7989f0ef6e87b5721c58737a06c38
  MUSL_CROSS_DIR=/opt/aarch64-linux-musl-cross
  curl -sSL "https://musl.cc/${MUSL_CROSS_TARBALL}" -o "/tmp/${MUSL_CROSS_TARBALL}"
  echo "${MUSL_CROSS_SHA256}  /tmp/${MUSL_CROSS_TARBALL}" | sha256sum -c -
  tar xz -C /opt -f "/tmp/${MUSL_CROSS_TARBALL}"
  rm "/tmp/${MUSL_CROSS_TARBALL}"
  export CC_aarch64_unknown_linux_musl=${MUSL_CROSS_DIR}/bin/aarch64-linux-musl-gcc
  export CXX_aarch64_unknown_linux_musl=${MUSL_CROSS_DIR}/bin/aarch64-linux-musl-g++
  export AR_aarch64_unknown_linux_musl=${MUSL_CROSS_DIR}/bin/aarch64-linux-musl-ar
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=${MUSL_CROSS_DIR}/bin/aarch64-linux-musl-gcc
else
  echo "Unsupported target platform: ${TARGETPLATFORM}"
  exit 1
fi

rustup target add "${RUST_TARGET}"
cargo build --release --features "$FEATURES" --target="${RUST_TARGET}"
cp "target/${RUST_TARGET}/release/df_engine" .
