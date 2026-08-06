import os
import subprocess
import sys

file_path = "rust/otap-dataflow/crates/core-nodes/src/exporters/otap_exporter/mod.rs"

# 1. Read file
with open(file_path, "r", encoding="utf-8") as f:
    content = f.read()

# 2. Locate enqueue_stream_batch bounds in mod.rs
start_marker = "    async fn enqueue_stream_batch("
end_marker = "    async fn drain_pdata_metrics_updates("

start_idx = content.find(start_marker)
end_idx = content.find(end_marker)

if start_idx == -1 or end_idx == -1:
    print("Error: Could not locate enqueue_stream_batch bounds in mod.rs")
    sys.exit(1)

# Extract and replace the whole function with the blocking version
reverted_func = """    async fn enqueue_stream_batch(
        &mut self,
        sender: &Sender<StreamBatch>,
        pdata: OtapPdata,
        message: OtapArrowRecords,
        pdata_metrics_rx: &mut Receiver<PDataMetricsUpdate>,
        effect_handler: &local::EffectHandler<OtapPdata>,
        msg_chan: &mut ExporterInbox<OtapPdata>,
    ) -> Result<EnqueueResult, Error> {
        let queue_depth = sender.max_capacity() - sender.capacity();
        self.async_metrics
            .stream_enqueue_depth
            .record(queue_depth as f64);
        let enqueue_start = Instant::now();
        
        match sender.reserve().await {
            Ok(permit) => {
                permit.send((pdata, message));
                self.async_metrics
                    .stream_enqueue_duration_ns
                    .record(elapsed_nanos(enqueue_start));
                Ok(EnqueueResult::Done)
            }
            Err(_) => {
                self.async_metrics
                    .stream_enqueue_duration_ns
                    .record(elapsed_nanos(enqueue_start));
                Ok(EnqueueResult::Done)
            }
        }
    }

"""

new_content = content[:start_idx] + reverted_func + content[end_idx:]

with open(file_path, "w", encoding="utf-8") as f:
    f.write(new_content)

print("1. Reverted enqueue_stream_batch to blocking implementation.")

try:
    print("\n2. Running deadlock test (Should FAIL/TIMEOUT now)...")
    subprocess.run([
        "cargo", "test", 
        "--manifest-path", "rust/otap-dataflow/Cargo.toml", 
        "--package", "otap-df-core-nodes", 
        "--lib", "exporters::otap_exporter::tests::test_otap_exporter_deadlock_on_full_queue_shutdown"
    ], check=True)
except subprocess.CalledProcessError:
    print("\nTest failed (deadlock occurred and timed out) as expected!")
finally:
    print("\n3. Restoring mod.rs to fixed version...")
    subprocess.run(["git", "checkout", "--", file_path], check=True)

print("\n4. Running deadlock test again (Should PASS now)...")
subprocess.run([
    "cargo", "test", 
    "--manifest-path", "rust/otap-dataflow/Cargo.toml", 
    "--package", "otap-df-core-nodes", 
    "--lib", "exporters::otap_exporter::tests::test_otap_exporter_deadlock_on_full_queue_shutdown"
], check=True)
