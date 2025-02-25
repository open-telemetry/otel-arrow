use std::sync::Mutex;

use async_trait::async_trait;
use futures::{
    future::{select, Either},
    pin_mut,
};
use once_cell::sync::Lazy;
use receiver::{
    signal::{Signal, SignalReceiver},
    AsyncReceiver, EngineHandler, Error, ReceiverFactory,
};
use serde_yaml::Value;
use tokio::io::AsyncReadExt;

use crate::common::Message;

pub static COUNTERS: Lazy<Mutex<Counters>> = Lazy::new(|| Mutex::new(Counters::default()));

/// A set of counters for the integration tests.
#[derive(Default, Debug)]
pub struct Counters {
    pub init_count: usize,
    pub receive_count: usize,
    pub stop_count: usize,
}

pub struct TestReceiver {
    tcp_port: u16,
}

#[async_trait]
impl AsyncReceiver<Message> for TestReceiver {
    async fn init(&mut self, engine_handler: EngineHandler) -> Result<(), Error> {
        COUNTERS.lock().expect("lock failed").init_count += 1;
        self.tcp_port = engine_handler.context().get_value::<u16>("receiver_tcp_port", 50000);
        Ok(())
    }

    async fn receive(
        &mut self,
        mut signal_receiver: SignalReceiver,
        effect_handler: beaubourg::receiver::effect::EffectHandler<Message>,
    ) -> Result<(), beaubourg::receiver::Error> {
        COUNTERS.lock().expect("lock failed").receive_count += 1;
        let addr: std::net::SocketAddr = format!("0.0.0.0:{}", self.tcp_port).parse().unwrap();
        let listener = effect_handler.tokio_tcp_listener(addr).await?;

        tracing::info!(thread_id=?std::thread::current().id(), address=%addr, "Ready to accept connection", );

        loop {
            let signal_future = signal_receiver.recv();
            let accept_future = listener.accept();

            pin_mut!(signal_future, accept_future);

            match select(signal_future, accept_future).await {
                Either::Left((signal, _)) => {
                    match signal {
                        Signal::Stop => tracing::info!("Receiver stopped"),
                        _ => panic!("Unexpected signal {:?}", signal),
                    }
                    break;
                }
                Either::Right((socket, _)) => match socket {
                    Ok((mut socket, _)) => {
                        let mut buf = vec![0; 1024];

                        match socket.read(&mut buf).await {
                            Ok(n) => {
                                if n == 0 {
                                    continue;
                                }

                                match std::str::from_utf8(&buf[0..n]) {
                                    Ok(v) => {
                                        if let Err(e) = effect_handler
                                            .send_messages(vec![Message {
                                                origin: format!("thread-{:?}", std::thread::current().id()),
                                                payload: v.into(),
                                            }])
                                            .await
                                        {
                                            tracing::error!("Error sending message: {:?}", e);
                                        }

                                        if v == "quit" {
                                            tracing::info!("quitting");
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("utf8 error: {:?}", e);
                                        panic!("Invalid UTF-8 sequence: {}", e);
                                    }
                                };
                            }
                            Err(err) => {
                                tracing::error!("Error reading from socket: {:?}", err);
                                panic!("Error reading from socket: {:?}", err);
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!("Error accepting socket: {:?}", err);
                        panic!("accept failed {}", err);
                    }
                },
            }
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        COUNTERS.lock().expect("lock failed").stop_count += 1;
        Ok(())
    }
}

#[derive(Default)]
pub struct TestReceiverFactory {}

impl ReceiverFactory<Message> for TestReceiverFactory {
    fn create(
        &self,
        receiver_name: &str,
        receiver_type: &str,
        _config: Value,
    ) -> Result<Box<dyn AsyncReceiver<Message> + Send + Sync>, beaubourg::receiver::Error> {
        match receiver_type {
            "test" => {
                let receiver = Box::new(TestReceiver { tcp_port: 0 });

                Ok(receiver as Box<dyn AsyncReceiver<Message> + Send + Sync>)
            }
            _ => Err(beaubourg::receiver::Error::UnknownReceiver {
                receiver: receiver_name.into(),
                r#type: receiver_type.into(),
            }),
        }
    }
}
