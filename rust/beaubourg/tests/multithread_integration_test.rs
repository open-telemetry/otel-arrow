extern crate core;

use color_eyre::eyre::Result;
use context::{Context, Value};
use engine::{multi_threaded, Engine, Error};
use task::labels::ProcessLabels;

use crate::{
    common::{async_run_tcp_client, available_tcp_ports, init, run_tcp_server},
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
impl engine::AsyncObserver for TestController {
    async fn on_started(&self) {
        async_run_tcp_client(self.msg_count, self.receiver_tcp_port).await;
    }

    async fn on_stopped(&self) {}
}

/// Starts the multi-threaded engine test (see configuration file for a
/// description of the pipeline).
///
/// Expected outcome: the number of messages received by the exporter should be
/// equal to the number of messages sent.
#[test]
fn multithread_engine() -> Result<(), Error> {
    init().expect("Failed to initialize tracing");

    let msg_count = 10;
    let (receiver_tcp_port, exporter_tcp_port) = available_tcp_ports(50050);
    let mut context = Context::default();
    context.set("receiver_tcp_port", Value::I64(receiver_tcp_port as i64));
    context.set("exporter_tcp_port", Value::I64(exporter_tcp_port as i64));

    // Initialize the engine with 3 custom factories.
    let mut engine = multi_threaded::Engine::with_context(
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
    {
        // Expected number of call to the init method = number of cores.
        // Expected number of call to the process method = number of cores.
        // Expected number of call to the stop method = number of cores.
        let counters = receiver::COUNTERS.lock().expect("Failed to lock receiver counters");
        assert_eq!(counters.init_count, 1);
        assert_eq!(counters.receive_count, 1);
        assert_eq!(counters.stop_count, 1);
    }
    {
        // Expected number of processors = number of cores.
        // Expected number of call to the init method = number of cores.
        // Expected number of call to the process method = msg_count.
        // Expected number of call to the stop method = number of cores.
        let counters = processor::COUNTERS.lock().expect("Failed to lock processor counters");
        assert_eq!(counters.init_count, 1);
        assert_eq!(counters.process_count, msg_count);
        assert_eq!(counters.stop_count, 1);
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
