// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Run DHAT trials without concurrent test-harness allocations.

pub fn run(name: &'static str, test: fn()) {
    let mut args = libtest_mimic::Arguments::from_args();
    // libtest-mimic 0.8.2 runs trials on the calling thread for Some(1).
    // Enforce this after parsing: worker threads would contaminate global DHAT stats.
    args.test_threads = Some(1);
    libtest_mimic::run(
        &args,
        vec![libtest_mimic::Trial::test(name, move || {
            test();
            Ok(())
        })],
    )
    .exit();
}

pub fn profiler(target: &str, test: &str) -> dhat::Profiler {
    // Construct the path before profiling so its allocation is not measured.
    let path = format!("dhat-{target}-{test}-{}.json", std::process::id());
    dhat::Profiler::builder().testing().file_name(path).build()
}
