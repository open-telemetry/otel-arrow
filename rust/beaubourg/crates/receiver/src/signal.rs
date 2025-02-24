//! Signals for receivers.

use std::{
    fmt::{Display, Formatter},
    time::{Duration, Instant},
};

use futures::{
    future::{select, Either},
    pin_mut,
};
use tokio::sync::mpsc::Receiver;

/// The different signals from Beaubourg that can be received by a receiver.
#[derive(Debug)]
#[non_exhaustive]
pub enum Signal {
    /// Signal triggered by a timer.
    TimerTick {
        /// Timestamp of the signal.
        instant: Instant,
        /// Origin of the timer.
        timer_source: usize,
    },

    /// Signal received when the receiver must stop.
    Stop,
}

/// Set of methods exposed by a signal.
impl Signal {
    /// Returns the name of this signal.
    pub fn name(&self) -> &'static str {
        match self {
            Signal::TimerTick { .. } => "TimerTick",
            Signal::Stop => "Stop",
        }
    }
}

impl Display for Signal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// A signal receiver that can be used to receive signals transmitted by
/// Beaubourg to a receiver.
pub enum SignalReceiver {
    /// A signal receiver that can receive signals from a mpsc channel.
    Receiver {
        /// The underlying receiver.
        receiver: Receiver<Signal>,
    },
    /// A signal receiver that can receive signals from a mpsc channel or a
    /// tokio timer.
    ReceiverTimer {
        /// The underlying receiver.
        receiver: Receiver<Signal>,
        /// The origin of the timer.
        timer_source: usize,
        /// The interval of the timer.
        interval: tokio::time::Interval,
    },
}

impl SignalReceiver {
    /// Creates a new receiver that can be used to receive signals from the
    /// exporter.
    pub fn with_receiver(receiver: Receiver<Signal>) -> Self {
        SignalReceiver::Receiver { receiver }
    }

    /// Creates a new receiver that can be used to receive signals from
    /// Beaubourg.
    pub fn with_receiver_timer(receiver: Receiver<Signal>, duration: Duration, timer_source: usize) -> Self {
        SignalReceiver::ReceiverTimer {
            receiver,
            timer_source,
            interval: tokio::time::interval(duration),
        }
    }

    /// Returns the next signal to process by the receiver.
    pub async fn recv(&mut self) -> Signal {
        match self {
            SignalReceiver::Receiver { receiver } => match receiver.recv().await {
                Some(signal) => signal,
                None => Signal::Stop,
            },
            SignalReceiver::ReceiverTimer {
                receiver,
                interval,
                timer_source,
            } => {
                let interval_fut = interval.tick();
                let receiver_fut = receiver.recv();

                pin_mut!(interval_fut, receiver_fut);

                match select(interval_fut, receiver_fut).await {
                    Either::Left((instant, _)) => Signal::TimerTick {
                        instant: instant.into_std(),
                        timer_source: *timer_source,
                    },
                    Either::Right((signal, _)) => match signal {
                        Some(signal) => signal,
                        None => Signal::Stop,
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::signal::{Signal, SignalReceiver};

    #[tokio::test]
    async fn test() {
        let (sender, receiver) = tokio::sync::mpsc::channel(10);
        let mut signal_receiver = SignalReceiver::with_receiver_timer(receiver, Duration::from_secs(1), 0);

        sender.send(Signal::Stop).await.unwrap();
        sender.send(Signal::Stop).await.unwrap();

        // Should receive the first stop signal.
        match signal_receiver.recv().await {
            Signal::TimerTick { .. } => panic!("unexpected signal"),
            Signal::Stop => { /* expected signal */ }
        }

        // Should receive the second stop signal.
        match signal_receiver.recv().await {
            Signal::TimerTick { .. } => panic!("unexpected signal"),
            Signal::Stop => { /* expected signal */ }
        }

        // Should receive the first timer tick signal (as there is no other signal to
        // consume).
        match signal_receiver.recv().await {
            Signal::TimerTick { .. } => { /* expected signal */ }
            Signal::Stop => panic!("unexpected signal"),
        }
    }
}
