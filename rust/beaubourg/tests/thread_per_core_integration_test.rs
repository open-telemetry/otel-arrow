extern crate core;

use color_eyre::eyre::Result;
use context::{Context, Value};
use engine::{thread_per_core, Engine, Error};
use task::labels::ProcessLabels;

use crate::{
    common::{available_tcp_ports, init, run_tcp_client, run_tcp_server},
    exporter::TestExporterFactory,
    processor::TestProcessorFactory,
    receiver::TestReceiverFactory,
};

mod common;
mod exporter;
mod processor;
mod receiver;

struct TestController {
    msg_count: usize,
    receiver_tcp_port: u16,
}

impl TestController {
    pub fn new(msg_count: usize, receiver_tcp_port: u16) -> Self {
        Self {
            msg_count,
            receiver_tcp_port,
        }
    }
}

#[async_trait::async_trait]
impl engine::Observer for TestController {
    fn on_started(&self) {
        run_tcp_client(self.msg_count, self.receiver_tcp_port);
    }
}

/// Starts the Thread Per Core engine test (see configuration file for a
/// description of the pipeline).
/// - 1 pipeline instance is created per core.
/// - 1 receiver with a TCP listener listening on a random port (mode
///   REUSE_ADDRESS, REUSE_PORT).
/// - 1 NOOP processor.
/// - 1 exporter counting the messages received.
///
/// Expected outcome: the number of messages received by the exporter should be
/// equal to the number of messages sent.
#[test]
fn thread_per_core_engine() -> Result<(), Error> {
    init().expect("Failed to initialize tracing");

    let msg_count = 10;
    let (receiver_tcp_port, exporter_tcp_port) = available_tcp_ports(50060);
    let mut context = Context::default();
    context.set("receiver_tcp_port", Value::I64(receiver_tcp_port as i64));
    context.set("exporter_tcp_port", Value::I64(exporter_tcp_port as i64));

    // Initialize the engine with 3 custom factories.
    let mut engine = thread_per_core::Engine::with_context(
        TestReceiverFactory::default(),
        TestProcessorFactory::default(),
        TestExporterFactory::default(),
        context,
    );

    // Start TCP test server to receive messages produced by the pipeline.
    let join_handle = run_tcp_server(msg_count, exporter_tcp_port, engine.command_handler());
    engine.observer(TestController::new(msg_count, receiver_tcp_port));

    // Starts the engine with a custom process label and a custom configuration.
    engine.run(ProcessLabels::new("test"), "tests/config.yaml")?;

    join_handle.join().unwrap();

    // General assertions.
    let cpu_count = num_cpus::get();
    {
        // Expected number of processors = number of cores.
        // Expected number of call to the init method = number of cores.
        // Expected number of call to the process method = number of cores.
        // Expected number of call to the stop method = number of cores.
        let counters = receiver::COUNTERS.lock().expect("Failed to lock receiver counters");
        assert_eq!(counters.init_count, cpu_count);
        assert_eq!(counters.receive_count, cpu_count);
        assert_eq!(counters.stop_count, cpu_count);
    }
    {
        // Expected number of processors = number of cores.
        // Expected number of call to the init method = number of cores.
        // Expected number of call to the process method = msg_count.
        // Expected number of call to the stop method = number of cores.
        let counters = processor::COUNTERS.lock().expect("Failed to lock processor counters");
        assert_eq!(counters.init_count, cpu_count);
        assert_eq!(counters.process_count, msg_count);
        assert_eq!(counters.stop_count, cpu_count);
    }
    {
        let counters = exporter::COUNTERS.lock().expect("Failed to lock exporter counters");
        assert_eq!(
            counters.init_count,
            1 /* test/0 singleton */ + 2 * num_cpus::get() /* test/1 task_per_core=2 */
        );
        assert_eq!(
            counters.export_count,
            1 /* test/0 singleton */ + 2 * num_cpus::get() /* test/1 task_per_core=2 */
        );
        assert_eq!(
            counters.stop_count,
            1 /* test/0 singleton */ + 2 * num_cpus::get() /* test/1 task_per_core=2 */
        );
    }

    Ok(())
}
