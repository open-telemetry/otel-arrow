#![deny(
    trivial_numeric_casts,
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unused_extern_crates,
    unused_results
)]
#![warn(rust_2021_compatibility, unreachable_pub)]

//! A signal is either an TimerTick signal (triggered by a configured timer),
//! or a batch of messages signal. An EventReceiver is the interface through
//! which an exporter can receive new signals from the engine.

use std::{
    fmt::{Display, Formatter},
    time::{Duration, Instant},
};

use futures::{
    future::{select, Either},
    pin_mut,
};

/// Engine signal that can be received from a processor or an exporter.
#[derive(Debug)]
#[non_exhaustive]
pub enum Signal<Msg: 'static + Clone + Send> {
    /// Signal variant triggered by a timer.
    TimerTick {
        /// Timestamp of the signal.
        instant: Instant,
        /// Origin of the timer (could be a processor or an exporter).
        timer_source: usize,
    },
    /// Signal variant triggered by the reception of a batch of messages.
    Messages {
        /// Messages received.
        messages: Vec<Msg>,
    },
    /// Signal variant received when the processor or exporter must stop.
    Stop,
}

/// Set of methods exposed by a signal.
impl<Msg> Signal<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Returns the name of this signal.
    pub fn name(&self) -> &'static str {
        match self {
            Signal::TimerTick { .. } => "TimerTick",
            Signal::Messages { .. } => "Messages",
            Signal::Stop => "Stop",
        }
    }
}

impl<Msg> Display for Signal<Msg>
where
    Msg: 'static + Clone + Send,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// A receiver that can be used to receive signals from the exporter.
pub enum SignalReceiver<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// A signal receiver with only a tokio::sync::mpsc::Receiver.
    Receiver {
        /// A tokio mpsc receiver.
        receiver: flume::Receiver<Vec<Msg>>,
    },
    /// A signal receiver with a tokio::sync::mpsc::Receiver and a tokio
    /// interval timer.
    ReceiverInterval {
        /// A tokio mpsc receiver.
        receiver: flume::Receiver<Vec<Msg>>,
        /// The origin of the timer.
        timer_source: usize,
        /// The interval of the timer.
        interval: tokio::time::Interval,
    },
}

impl<Msg> SignalReceiver<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Creates a new receiver that can be used to receive signals from the
    /// exporter.
    pub fn with_receiver(receiver: flume::Receiver<Vec<Msg>>) -> Self {
        SignalReceiver::Receiver { receiver }
    }

    /// Creates a new receiver that can be used to receive signals from the
    /// exporter. The receiver will also receive signal::TimerTick signal
    /// every `duration`.
    pub fn with_receiver_timer(receiver: flume::Receiver<Vec<Msg>>, duration: Duration, timer_source: usize) -> Self {
        SignalReceiver::ReceiverInterval {
            receiver,
            timer_source,
            interval: tokio::time::interval(duration),
        }
    }

    /// Returns the next signal to process by the exporter.
    pub async fn recv(&mut self) -> Signal<Msg> {
        match self {
            SignalReceiver::Receiver { receiver } => match receiver.recv_async().await {
                Ok(messages) => Signal::Messages { messages },
                Err(err) => {
                    tracing::warn!("Error receiving messages (Flume MPMC channel): {}", err);
                    Signal::Stop
                }
            },
            SignalReceiver::ReceiverInterval {
                receiver,
                interval,
                timer_source,
            } => {
                let interval_fut = interval.tick();
                let receiver_fut = receiver.recv_async();

                pin_mut!(interval_fut, receiver_fut);

                match select(interval_fut, receiver_fut).await {
                    Either::Left((instant, _)) => Signal::TimerTick {
                        instant: instant.into_std(),
                        timer_source: *timer_source,
                    },
                    Either::Right((messages, _)) => match messages {
                        Ok(messages) => Signal::Messages { messages },
                        Err(err) => {
                            tracing::warn!("Error receiving messages (Flume MPMC channel): {}", err);
                            Signal::Stop
                        }
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::{Signal, SignalReceiver};

    #[tokio::test]
    async fn test() {
        let (sender, receiver) = flume::bounded(10);
        let mut message_stream = SignalReceiver::with_receiver_timer(receiver, Duration::from_secs(1), 0);

        sender.send_async(vec!["test1".to_string()]).await.unwrap();
        sender.send_async(vec!["test2".to_string()]).await.unwrap();

        match message_stream.recv().await {
            Signal::TimerTick { .. } => panic!("TimerTick signal received before messages"),
            Signal::Messages { messages } => {
                assert_eq!(messages.len(), 1);
                assert_eq!(messages[0], "test1".to_string());
            }
            Signal::Stop => panic!("Stop signal received before messages"),
        }

        match message_stream.recv().await {
            Signal::TimerTick { .. } => panic!("TimerTick signal received before messages"),
            Signal::Messages { messages } => {
                assert_eq!(messages.len(), 1);
                assert_eq!(messages[0], "test2".to_string());
            }
            Signal::Stop => panic!("Stop signal received before messages"),
        }

        match message_stream.recv().await {
            Signal::TimerTick { .. } => { /*OK*/ }
            Signal::Messages { .. } => panic!("MessagesReceived signal received after all messages consumed"),
            Signal::Stop => panic!("Stop signal received before messages"),
        }
    }
}
