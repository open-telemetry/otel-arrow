// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    config::{CompiledConfig, Config, literal_include_target},
    delivery::{Correlation, send_attempt},
    runtime_worker::{self, Command},
    startup::{StartupError, WorkerInputs},
    workers::{self, WorkerOutcome, Workers},
};
use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::{
    ReceiverFactory,
    control::NodeControlMsg,
    error::{Error, ReceiverErrorKind},
    local::receiver as local,
    receiver::ReceiverWrapper,
    state_dir::StateDirectory,
    terminal_state::TerminalState,
    wiring_contract::WiringContract,
};
use otel_arrow_dfe_filelog_checkpoint::store::StateRoot;
use otel_arrow_dfe_otap::{OTAP_RECEIVER_FACTORIES, pdata::OtapPdata};
use std::{
    num::NonZeroUsize,
    os::fd::AsFd,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::oneshot;

struct Receiver {
    config: CompiledConfig,
    root: StateDirectory,
    file: PathBuf,
}

fn invalid(message: impl ToString) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: message.to_string(),
    }
}

fn compile(
    value: &serde_json::Value,
    group: &str,
    pipeline: &str,
    node: &str,
) -> Result<(CompiledConfig, PathBuf), ConfigError> {
    let config = Config::from_json(value.clone()).map_err(invalid)?;
    let compiled = config
        .compile(group, pipeline, node, Path::new(""))
        .map_err(invalid)?;
    let config = compiled.inputs().settings();
    if config.follow_symlinks || config.include.len() != 1 {
        return Err(invalid(
            "experimental filelog requires one absolute literal file and follow_symlinks: false",
        ));
    }
    let file = literal_include_target(&compiled.paths().include_roots()[0], &config.include[0])
        .ok_or_else(|| invalid("experimental filelog requires a literal include path"))?;
    Ok((compiled, file))
}

#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
static FILELOG_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: "urn:otel:receiver:filelog",
    create: |pipeline, node, user, receiver, _| {
        if pipeline.num_cores() != 1 || user.outputs.len() > 1 {
            return Err(invalid("filelog requires one core and one direct output"));
        }
        let root = pipeline
            .state_directory()
            .cloned()
            .ok_or_else(|| invalid("filelog requires a durable state directory"))?;
        let (config, file) = compile(
            &user.config,
            pipeline.pipeline_group_id().as_ref(),
            pipeline.pipeline_id().as_ref(),
            receiver.name.as_ref(),
        )?;
        Ok(
            ReceiverWrapper::local(Receiver { config, root, file }, node, user, receiver)
                .with_startup_readiness(),
        )
    },
    context_declarations: None,
    wiring_contract: WiringContract {
        requires_ack_propagation: true,
        ..WiringContract::at_most_per_output(1)
    },
    validate_config: |value| compile(value, "group", "pipeline", "filelog").map(drop),
};

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for Receiver {
    async fn start(
        self: Box<Self>,
        mut control: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let error = |error: String| Error::ReceiverError {
            receiver: effects.receiver_id(),
            kind: ReceiverErrorKind::Other,
            error,
            source_detail: String::new(),
        };
        let Receiver { root, config, file } = *self;
        let drain_timeout = config.inputs().settings().drain_timeout;
        let mut workers = Workers::new(config.inputs().settings().worker_shutdown_timeout)
            .map_err(|e| error(e.to_string()))?;
        let cancel = workers.cancellation();
        let (observations, input, mut output) = workers::channels(
            NonZeroUsize::MIN,
            NonZeroUsize::MIN,
            cancel.clone(),
            cancel.clone(),
        );
        let (scanner_tx, scanner_rx) = oneshot::channel::<runtime_worker::ScannerSetup>();
        let (ready_tx, mut ready) = oneshot::channel();
        let (done_tx, mut done) = oneshot::channel();
        let budget = Arc::default();
        let scan_budget = Arc::clone(&budget);
        workers
            .start_discovery(move |cancel| {
                if let Ok((scanner, interval)) = scanner_rx.blocking_recv() {
                    runtime_worker::discover(
                        scanner,
                        observations,
                        scan_budget,
                        cancel,
                        interval,
                        &file,
                    );
                }
            })
            .map_err(|e| error(e.to_string()))?;
        #[cfg(test)]
        let probe = super::runtime_tests::probe();
        #[cfg(test)]
        if let Some(p) = &probe {
            p.register(output.test_commands());
        }
        workers
            .start_read_checkpoint(move |cancel| {
                let result = (|| {
                    let root = StateRoot::from_borrowed_fd(root.as_fd())?;
                    let startup = WorkerInputs::open(config, &root, &cancel)?;
                    runtime_worker::run(
                        startup,
                        input,
                        scanner_tx,
                        ready_tx,
                        budget,
                        cancel,
                        #[cfg(test)]
                        probe,
                    )
                })();
                let _ = done_tx.send(result);
            })
            .map_err(|e| error(e.to_string()))?;
        let mut starting = true;
        let mut draining = false;
        let mut send = std::pin::pin!(None);
        let mut sending: Option<Correlation> = None;
        let mut early_reply = None;
        let (deadline_tx, deadline_rx) = oneshot::channel();
        let mut deadline_tx = Some(deadline_tx);
        let reactor = async {
            loop {
                tokio::select! {
                    biased;
                    _ = cancel.cancelled() => {
                        if draining { break; }
                        return Err(error("filelog worker stopped".to_owned()));
                    }
                    message = control.recv() => {
                        let (data, ack) = match message? {
                            NodeControlMsg::Shutdown { .. } => { draining = false; break; },
                            NodeControlMsg::DrainIngress { deadline, .. } => {
                                if let Some(tx) = deadline_tx.take() {
                                    draining = true;
                                    let local = std::time::Instant::now().checked_add(drain_timeout).unwrap_or(deadline);
                                    let _ = tx.send(deadline.min(local));
                                    output.command(Command::Drain).await.map_err(|e| error(e.to_string()))?;
                                    #[cfg(test)]
                                    if let Some(p) = super::runtime_tests::probe() {
                                        p.drain_event(super::runtime_tests::DrainEvent::Queued);
                                    }
                                }
                                continue;
                            }
                            NodeControlMsg::Ack(ack) => (ack.unwind.route.calldata, true),
                            NodeControlMsg::Nack(nack) => (nack.unwind.route.calldata, false),
                            _ => continue,
                        };
                        if let Some(id) = sending {
                            // The worker must see send acceptance before the matching completion.
                            if data == id.calldata() && early_reply.is_none() {
                                early_reply = Some(Command::Completion(data, ack));
                            }
                        } else {
                            output.command(Command::Completion(data, ack)).await
                                .map_err(|e| error(e.to_string()))?;
                        }
                    }
                    ready_result = &mut ready, if starting => {
                        ready_result.map_err(|_| error("read worker failed during startup".to_owned()))?;
                        starting = false;
                        effects.signal_ready();
                    }
                    outcome = async {
                        send.as_mut().as_pin_mut().expect("pending publication").await
                    }, if sending.is_some() => {
                        let id = sending.take().expect("pending publication");
                        send.set(None);
                        output.command(Command::Sent(id, outcome?)).await
                            .map_err(|e| error(e.to_string()))?;
                        if let Some(reply) = early_reply.take() {
                            output.command(reply).await.map_err(|e| error(e.to_string()))?;
                        }
                    }
                    batch = output.batch(), if !starting && sending.is_none() => {
                        let attempt = match batch {
                            Ok(attempt) => attempt,
                            Err(_) if draining => break,
                            Err(e) => return Err(error(e.to_string())),
                        };
                        sending = Some(attempt.correlation);
                        send.set(Some(send_attempt(&effects, attempt, cancel.cancelled())));
                    }
                }
            }
            Ok(())
        };
        // Bound the whole drain, including a blocked worker command or downstream send.
        let result = {
            tokio::pin!(reactor);
            tokio::select! {
                result = &mut reactor => result,
                Ok(deadline) = deadline_rx => {
                    tokio::time::timeout_at(deadline.into(), reactor).await
                        .unwrap_or_else(|_| Err(error("filelog delivery drain timed out".to_owned())))
                }
            }
        };
        let _ = cancel.cancel();
        let report = workers.join().await;
        if [&report.read_checkpoint, &report.discovery]
            .into_iter()
            .any(|r| {
                !matches!(
                    r,
                    Some(WorkerOutcome::Joined {
                        deadline_missed: false
                    })
                )
            })
        {
            return Err(error(format!("filelog worker shutdown: {report:?}")));
        }
        // Once joined the result is available even if worker exit and channel closure raced.
        let worker_result = done.try_recv().map_err(|e| error(e.to_string()))?;
        let completed_drain = matches!(worker_result, Ok(runtime_worker::WorkerExit::Drained));
        if let Err(e) = worker_result {
            let expected_shutdown = e.downcast_ref::<workers::ChannelError>()
                == Some(&workers::ChannelError::Cancelled)
                || matches!(
                    e.downcast_ref::<StartupError>(),
                    Some(StartupError::Worker(workers::ChannelError::Cancelled))
                );
            if !expected_shutdown {
                return Err(error(e.to_string()));
            }
        }
        result?;
        if draining {
            if !completed_drain {
                return Err(error(
                    "filelog worker stopped before completing drain".to_owned(),
                ));
            }
            effects.notify_receiver_drained().await?;
        }
        Ok(TerminalState::default())
    }
}
