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
  if [ "${BUILDPLATFORM}" = "linux/arm64" ]; then
    # Native arm64 builder: use the musl.cc arm64-hosted native toolchain.
    # musl-tools/musl-gcc is avoided because its unprefixed compiler name causes
    # jemalloc's configure to misdetect the build as cross-compilation
    # (build=aarch64-linux-gnu vs host=aarch64-linux-musl), which breaks atomic
    # detection. The musl.cc native toolchain provides a properly-prefixed
    # aarch64-linux-musl-gcc that runs natively on arm64 with musl headers.
    MUSL_NATIVE_TARBALL=aarch64-linux-musl-native.tgz
    MUSL_NATIVE_SHA256=daf336cafa2c3c7daf42f6a46edc960f10a181fcf15ab9f1c43b192e8ad2a069
    MUSL_NATIVE_DIR=/opt/aarch64-linux-musl-native
    curl -sSfL "https://musl.cc/${MUSL_NATIVE_TARBALL}" -o "/tmp/${MUSL_NATIVE_TARBALL}"
    echo "${MUSL_NATIVE_SHA256}  /tmp/${MUSL_NATIVE_TARBALL}" | sha256sum -c -
    tar xz -C /opt -f "/tmp/${MUSL_NATIVE_TARBALL}"
    rm "/tmp/${MUSL_NATIVE_TARBALL}"
    # The native tarball ships aarch64-linux-musl-gcc-ar but not aarch64-linux-musl-ar.
    # cmake-based crates (e.g. zstd-sys) look for the latter by name, so symlink it.
    ln -s "${MUSL_NATIVE_DIR}/bin/aarch64-linux-musl-gcc-ar" "${MUSL_NATIVE_DIR}/bin/aarch64-linux-musl-ar"
    export CC_aarch64_unknown_linux_musl=${MUSL_NATIVE_DIR}/bin/aarch64-linux-musl-gcc
    export CXX_aarch64_unknown_linux_musl=${MUSL_NATIVE_DIR}/bin/aarch64-linux-musl-g++
    export AR_aarch64_unknown_linux_musl=${MUSL_NATIVE_DIR}/bin/aarch64-linux-musl-ar
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=${MUSL_NATIVE_DIR}/bin/aarch64-linux-musl-gcc
  elif [ "${BUILDPLATFORM}" = "linux/amd64" ]; then
    # Cross-compile (amd64 -> arm64): Debian's gcc-aarch64-linux-gnu uses glibc
    # system headers; glibc >= 2.38 maps _GNU_SOURCE to _ISOC23_SOURCE, which
    # redirects strtol/sscanf to __isoc23_strtol/__isoc23_sscanf. Those C23
    # symbols don't exist in musl libc, causing link failures in C dependencies
    # that define _GNU_SOURCE (e.g. aws-lc-sys/bcm.c). Use the musl.cc
    # x86_64-hosted cross-toolchain which ships its own musl headers instead.
    MUSL_CROSS_TARBALL=aarch64-linux-musl-cross.tgz
    MUSL_CROSS_SHA256=c909817856d6ceda86aa510894fa3527eac7989f0ef6e87b5721c58737a06c38
    MUSL_CROSS_DIR=/opt/aarch64-linux-musl-cross
    curl -sSfL "https://musl.cc/${MUSL_CROSS_TARBALL}" -o "/tmp/${MUSL_CROSS_TARBALL}"
    echo "${MUSL_CROSS_SHA256}  /tmp/${MUSL_CROSS_TARBALL}" | sha256sum -c -
    tar xz -C /opt -f "/tmp/${MUSL_CROSS_TARBALL}"
    rm "/tmp/${MUSL_CROSS_TARBALL}"
    export CC_aarch64_unknown_linux_musl=${MUSL_CROSS_DIR}/bin/aarch64-linux-musl-gcc
    export CXX_aarch64_unknown_linux_musl=${MUSL_CROSS_DIR}/bin/aarch64-linux-musl-g++
    export AR_aarch64_unknown_linux_musl=${MUSL_CROSS_DIR}/bin/aarch64-linux-musl-ar
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=${MUSL_CROSS_DIR}/bin/aarch64-linux-musl-gcc
  else
    echo "Unsupported build platform for arm64 target: ${BUILDPLATFORM}"
    exit 1
  fi
else
  echo "Unsupported target platform: ${TARGETPLATFORM}"
  exit 1
fi

rustup target add "${RUST_TARGET}"
cargo build --release --features "$FEATURES" --target="${RUST_TARGET}"
cp "target/${RUST_TARGET}/release/df_engine" .
