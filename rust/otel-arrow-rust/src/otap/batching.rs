//! Batching for `OtapArrowRecords`
//!
//!

use std::{num::NonZeroU64, time::Duration};

use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
        oneshot,
    },
    task::JoinHandle,
    time::{Instant, sleep},
};

use super::{OtapArrowRecords, error::Result, groups::RecordsGroup};

/// Control how we batch data
///
/// Specifically, how much data can we accumulate before we emit a batch, how long can we store that
/// data before emitting a batch, and what's the largest batch we'll emit.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Config {
    /// Equivalent to `send_batch_size` in the go implementation.
    size_trigger: NonZeroU64,

    /// Equivalent to `timeout` in the go implementation.
    time_trigger: Option<Duration>,

    /// Equivalent to `send_batch_max_size` in the go implementation.
    max_output_batch: Option<NonZeroU64>,
}

impl Config {
    /// Construct a new configuration struct
    #[must_use]
    pub fn new(
        size_trigger: NonZeroU64,
        time_trigger: Option<Duration>,
        max_output_batch: Option<NonZeroU64>,
    ) -> Self {
        Config {
            size_trigger,
            time_trigger,
            max_output_batch,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let size_trigger = NonZeroU64::new(8192).expect("this is fine");
        let time_trigger = Some(Duration::from_millis(200));
        let max_output_batch = None;
        Config {
            size_trigger,
            time_trigger,
            max_output_batch,
        }
    }
}

/// I only exist to conveniently group together data needed by `do_the_thing`.
struct BatcherState {
    config: Config,
    input_rx: UnboundedReceiver<OtapArrowRecords>, // but I'm keeping this one
    output_tx: UnboundedSender<OtapArrowRecords>,  // and I'm keeping this one too
    termination_rx: oneshot::Receiver<()>,
    current: Vec<OtapArrowRecords>,
}

/// I'm the user facing component
pub struct Batcher {
    input_tx: UnboundedSender<OtapArrowRecords>, // I give this one away
    output_rx: Option<UnboundedReceiver<OtapArrowRecords>>, // but I'm giving this one away
    // We use an Option here because you can't clone receivers, so we need the ability to note that
    // we don't have it anymore.
    termination_tx: Option<oneshot::Sender<()>>,
    _task_handle: JoinHandle<()>,
}

impl Batcher {
    /// How to send `OtapArrowRecords` for batching.
    #[must_use]
    pub fn inbound(&self) -> UnboundedSender<OtapArrowRecords> {
        self.input_tx.clone()
    }

    /// How to receive properly batched `OtapArrowRecords`
    pub fn outbound(&mut self) -> Option<UnboundedReceiver<OtapArrowRecords>> {
        self.output_rx.take()
    }
}

impl Batcher {
    /// Create a new `Batcher`!
    pub async fn new(config: Config) -> Self {
        let (input_tx, input_rx) = unbounded_channel();
        let (output_tx, output_rx) = unbounded_channel();
        let output_rx = Some(output_rx);
        let (termination_tx, termination_rx) = oneshot::channel();
        let termination_tx = Some(termination_tx);

        let current = Vec::new();
        let state = BatcherState {
            config,
            input_rx,
            output_tx,
            termination_rx,
            current,
        };
        let _task_handle = tokio::task::spawn(update_state(state));

        Self {
            input_tx,
            output_rx,
            termination_tx,
            _task_handle,
        }
    }
}

impl Drop for Batcher {
    fn drop(&mut self) {
        if let Some(tx) = self.termination_tx.take() {
            let _ = tx.send(());
        }
    }
}

struct Action {
    do_emit: bool,
    do_shutdown: bool,
}

fn size_triggers_emission(state: &BatcherState) -> bool {
    (state
        .current
        .iter()
        .map(|records| records.batch_length())
        .sum::<usize>() as u64)
        >= state.config.size_trigger.get()
}

/// Implement the batching state machine
///
/// There is one and only one place where we update state and respond to events: this function. We
/// rely on this simplifying assumption.
///
/// In a loop, we select against three async inputs:
/// * `input_rx`, representing new incoming data to batch
/// * a timer indicating whether our `config.time_trigger` has elapsed since last emission
/// * `termination_rx` indicating that the original `Batcher` has been dropped and that we should
///   exit the task
///
/// When new data arrives, we add it to `current` and then call `make_output_batches` and finally
/// update our timer. When the timer expires
async fn update_state(mut state: BatcherState) {
    let sleep = sleep(state.config.time_trigger.unwrap_or(Duration::MAX));
    tokio::pin!(sleep);

    loop {
        #[rustfmt::skip]
        let action = tokio::select! {
            _time_to_die = &mut state.termination_rx => {
                // Early termination: we exit regardless of what unprocessed data is lying around in
                // `state.current`.
                return;
            },

            incoming = state.input_rx.recv() => {
                match incoming {
                    Some(incoming) => {
                        state.current.push(incoming);
                        let do_emit = size_triggers_emission(&state);
                        let do_shutdown = false;
                        Action {do_emit, do_shutdown}
                    },
                    None => {
                        // The channel is closed, so we can never get any new batches. We can't
                        // necessarily shut down right away because `state.current` might still have
                        // stuff in it.
                        let do_emit = !state.current.is_empty();
                        let do_shutdown = true;
                        Action {do_emit, do_shutdown}
                    }
                }

            }

            _ = &mut sleep, if state.config.time_trigger.is_some() => {
                let do_emit = true;
                let do_shutdown = false;
                Action {do_emit, do_shutdown}
            }

            else => Action {do_emit: false, do_shutdown: false}
        };

        if action.do_emit {
            for records in make_output_batches(
                state.config.max_output_batch,
                std::mem::take(&mut state.current),
            )
            .expect("this is a bad place to recover from errors")
            {
                if state.output_tx.send(records).is_err() {
                    return;
                }
            }

            // Cancel any timer that is still running and reset it to `config.time_trigger` from
            // now.
            if let Some(duration) = state.config.time_trigger {
                sleep.as_mut().reset(Instant::now() + duration);
            }
        };

        if action.do_shutdown {
            // Handle deferred shutdown for when the channel gets closed.
            return;
        };
    }
}

fn make_output_batches(
    max_output_batch: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<Vec<OtapArrowRecords>> {
    // We have to deal with three complications here:
    // * batches that are too small
    // * batches that are too big
    // * cases where we have different types (logs/metrics/traces) intermingled

    // We deal with the last issue first, by splitting the input into three lists of the appropriate
    // types.
    let [mut logs, mut metrics, mut traces] = RecordsGroup::split_by_type(records);

    if let Some(max_output_batch) = max_output_batch {
        logs = logs.split(max_output_batch)?;
        metrics = metrics.split(max_output_batch)?;
        traces = traces.split(max_output_batch)?;
    }
    logs = logs.concatenate(max_output_batch)?;
    metrics = metrics.concatenate(max_output_batch)?;
    traces = traces.concatenate(max_output_batch)?;

    let mut result = Vec::new();
    result.extend(logs.into_otap_arrow_records());
    result.extend(metrics.into_otap_arrow_records());
    result.extend(traces.into_otap_arrow_records());

    // By splitting into 3 different lists, we've probably scrambled the ordering. We can't really
    // fix that problem in a general sense because each `OtapArrowRecords` will contain many rows ot
    // different times, but we can improve matters slightly by sorting on the smallest record time.

    // FIXME: sort here
    Ok(result)
}
