/// This is a port of the rdkafka ThreadedProducer and FutureProducer located at https://github.com/fede1024/rust-rdkafka/blob/master/src/producer/future_producer.rs.
/// This is done as a temporary workaround for the high CPU utilization issues seen related to the tight polling interval
/// in the main loop of the spawned thread. Longer term fixes might include:
/// 1. getting the upstream rdkafka project to implement this or similar changes (increase poll duration to 1s from 100ms).
/// 2. evaluating alternative kafka libraries.
/// 3. Others...
//
// ********** THREADED PRODUCER **********
//
use futures_channel::oneshot;
use futures::FutureExt;
use std::error::Error;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use rdkafka::client::{Client, DefaultClientContext, OAuthToken};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::ConsumerGroupMetadata;
use rdkafka::message::{DeliveryResult, OwnedHeaders, OwnedMessage};
use rdkafka::producer::future_producer::{Delivery, OwnedDeliveryResult};
use rdkafka::producer::{Producer, PurgeConfig};
use rdkafka::types::RDKafkaErrorCode;
use rdkafka::util::{AsyncRuntime, DefaultRuntime};
use rdkafka::{
    ClientConfig,
    config::{FromClientConfig, FromClientConfigAndContext},
    error::{KafkaError, KafkaResult},
    message::ToBytes,
    producer::{
        BaseProducer, BaseRecord, DefaultProducerContext, NoCustomPartitioner, Partitioner,
        ProducerContext,
    },
    util::Timeout,
};
use rdkafka::{ClientContext, Statistics, TopicPartitionList};
use rdkafka::{IntoOpaque, Message, Timestamp};
use tracing::{trace, warn};

/// A low-level Kafka producer with a separate thread for event handling.
///
/// The `ExporterThreadedProducer` is a [`BaseProducer`] with a separate thread
/// dedicated to calling `poll` at regular intervals in order to execute any
/// queued events, such as delivery notifications. The thread will be
/// automatically stopped when the producer is dropped.
#[must_use = "The threaded producer will stop immediately if unused"]
pub struct ExporterThreadedProducer<C, Part: Partitioner = NoCustomPartitioner>
where
    C: ProducerContext<Part> + 'static,
{
    producer: Arc<BaseProducer<C, Part>>,
    should_stop: Arc<AtomicBool>,
    handle: Option<Arc<JoinHandle<()>>>,
}

impl FromClientConfig for ExporterThreadedProducer<DefaultProducerContext, NoCustomPartitioner> {
    fn from_config(
        config: &ClientConfig,
    ) -> KafkaResult<ExporterThreadedProducer<DefaultProducerContext>> {
        ExporterThreadedProducer::from_config_and_context(config, DefaultProducerContext)
    }
}

impl<C, Part> FromClientConfigAndContext<C> for ExporterThreadedProducer<C, Part>
where
    Part: Partitioner + Send + Sync + 'static,
    C: ProducerContext<Part> + 'static,
{
    fn from_config_and_context(
        config: &ClientConfig,
        context: C,
    ) -> KafkaResult<ExporterThreadedProducer<C, Part>> {
        let producer = Arc::new(BaseProducer::from_config_and_context(config, context)?);
        let should_stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let producer = Arc::clone(&producer);
            let should_stop = should_stop.clone();
            thread::Builder::new()
                .name("producer polling thread".to_string())
                .spawn(move || {
                    trace!("Polling thread loop started");
                    loop {
                        // Running this in a tight loop results in non-negligable cpu utilization
                        // for each thread (1-2% of a core while idle). We increase the duration to 1 second, and sacrifice
                        // potential delay in the stop signal in exchange for much lower resource use.
                        producer.poll(Duration::from_millis(1000));
                        if should_stop.load(Ordering::Relaxed) {
                            // We received nothing and the thread should
                            // stop, so break the loop.
                            break;
                        }
                    }
                    trace!("Polling thread loop terminated");
                })
                .expect("Failed to start polling thread")
        };
        Ok(ExporterThreadedProducer {
            producer,
            should_stop,
            handle: Some(Arc::new(thread)),
        })
    }
}

impl<C, Part> ExporterThreadedProducer<C, Part>
where
    Part: Partitioner,
    C: ProducerContext<Part> + 'static,
{
    /// Sends a message to Kafka.
    ///
    /// See the documentation for [`BaseProducer::send`] for details.
    // Simplifying the return type requires generic associated types, which are
    // unstable.
    #[allow(clippy::result_large_err)]
    pub fn send<'a, K, P>(
        &self,
        record: BaseRecord<'a, K, P, C::DeliveryOpaque>,
    ) -> Result<(), (KafkaError, BaseRecord<'a, K, P, C::DeliveryOpaque>)>
    where
        K: ToBytes + ?Sized,
        P: ToBytes + ?Sized,
    {
        self.producer.send(record)
    }

    /// Polls the internal producer.
    ///
    /// This is not normally required since the `ExporterThreadedProducer` has a thread
    /// dedicated to calling `poll` regularly.
    #[allow(dead_code)]
    pub fn poll<T: Into<Timeout>>(&self, timeout: T) {
        self.producer.poll(timeout);
    }
}

impl<C, Part> Producer<C, Part> for ExporterThreadedProducer<C, Part>
where
    Part: Partitioner,
    C: ProducerContext<Part> + 'static,
{
    fn client(&self) -> &Client<C> {
        self.producer.client()
    }

    fn flush<T: Into<Timeout>>(&self, timeout: T) -> KafkaResult<()> {
        self.producer.flush(timeout)
    }

    fn purge(&self, flags: PurgeConfig) {
        self.producer.purge(flags)
    }

    fn in_flight_count(&self) -> i32 {
        self.producer.in_flight_count()
    }

    fn init_transactions<T: Into<Timeout>>(&self, timeout: T) -> KafkaResult<()> {
        self.producer.init_transactions(timeout)
    }

    fn begin_transaction(&self) -> KafkaResult<()> {
        self.producer.begin_transaction()
    }

    fn send_offsets_to_transaction<T: Into<Timeout>>(
        &self,
        offsets: &TopicPartitionList,
        cgm: &ConsumerGroupMetadata,
        timeout: T,
    ) -> KafkaResult<()> {
        self.producer
            .send_offsets_to_transaction(offsets, cgm, timeout)
    }

    fn commit_transaction<T: Into<Timeout>>(&self, timeout: T) -> KafkaResult<()> {
        self.producer.commit_transaction(timeout)
    }

    fn abort_transaction<T: Into<Timeout>>(&self, timeout: T) -> KafkaResult<()> {
        self.producer.abort_transaction(timeout)
    }
}

impl<C: ProducerContext + 'static> Clone for ExporterThreadedProducer<C> {
    fn clone(&self) -> Self {
        Self {
            producer: Arc::clone(&self.producer),
            should_stop: Arc::clone(&self.should_stop),
            handle: self.handle.clone(),
        }
    }
}

impl<C, Part> Drop for ExporterThreadedProducer<C, Part>
where
    Part: Partitioner,
    C: ProducerContext<Part> + 'static,
{
    fn drop(&mut self) {
        trace!("Destroy ExporterThreadedProducer");
        if let Some(handle) = self.handle.take().and_then(Arc::into_inner) {
            trace!("Stopping polling");
            self.should_stop.store(true, Ordering::Relaxed);
            trace!("Waiting for polling thread termination");
            match handle.join() {
                Ok(()) => trace!("Polling stopped"),
                Err(e) => warn!("Failure while terminating thread: {:?}", e),
            };
        }
        trace!("ExporterThreadedProducer destroyed");
    }
}

/// A record for the future producer.
///
/// Like [`BaseRecord`], but specific to the [`FutureProducer`]. The only
/// difference is that the [FutureRecord] doesn't provide custom delivery opaque
/// object.
#[derive(Debug)]
pub struct ExporterFutureRecord<'a, K: ToBytes + ?Sized, P: ToBytes + ?Sized> {
    /// Required destination topic.
    pub topic: &'a str,
    /// Optional destination partition.
    pub partition: Option<i32>,
    /// Optional payload.
    pub payload: Option<&'a P>,
    /// Optional key.
    pub key: Option<&'a K>,
    /// Optional timestamp.
    pub timestamp: Option<i64>,
    /// Optional message headers.
    pub headers: Option<OwnedHeaders>,
}

impl<'a, K: ToBytes + ?Sized, P: ToBytes + ?Sized> ExporterFutureRecord<'a, K, P> {
    /// Creates a new record with the specified topic name.
    pub fn to(topic: &'a str) -> ExporterFutureRecord<'a, K, P> {
        ExporterFutureRecord {
            topic,
            partition: None,
            payload: None,
            key: None,
            timestamp: None,
            headers: None,
        }
    }

    #[allow(dead_code)]
    fn from_base_record<D: IntoOpaque>(
        base_record: BaseRecord<'a, K, P, D>,
    ) -> ExporterFutureRecord<'a, K, P> {
        ExporterFutureRecord {
            topic: base_record.topic,
            partition: base_record.partition,
            key: base_record.key,
            payload: base_record.payload,
            timestamp: base_record.timestamp,
            headers: base_record.headers,
        }
    }

    /// Sets the destination partition of the record.
    #[allow(dead_code)]
    pub fn partition(mut self, partition: i32) -> ExporterFutureRecord<'a, K, P> {
        self.partition = Some(partition);
        self
    }

    /// Sets the destination payload of the record.
    pub fn payload(mut self, payload: &'a P) -> ExporterFutureRecord<'a, K, P> {
        self.payload = Some(payload);
        self
    }

    /// Sets the destination key of the record.
    pub fn key(mut self, key: &'a K) -> ExporterFutureRecord<'a, K, P> {
        self.key = Some(key);
        self
    }

    /// Sets the destination timestamp of the record.
    #[allow(dead_code)]
    pub fn timestamp(mut self, timestamp: i64) -> ExporterFutureRecord<'a, K, P> {
        self.timestamp = Some(timestamp);
        self
    }

    /// Sets the headers of the record.
    pub fn headers(mut self, headers: OwnedHeaders) -> ExporterFutureRecord<'a, K, P> {
        self.headers = Some(headers);
        self
    }

    fn into_base_record<D: IntoOpaque>(self, delivery_opaque: D) -> BaseRecord<'a, K, P, D> {
        BaseRecord {
            topic: self.topic,
            partition: self.partition,
            key: self.key,
            payload: self.payload,
            timestamp: self.timestamp,
            headers: self.headers,
            delivery_opaque,
        }
    }
}

/// A producer that returns a [`Future`] for every message being produced.
///
/// Since message production in rdkafka is asynchronous, the caller cannot
/// immediately know if the delivery of the message was successful or not. The
/// FutureProducer provides this information in a [`Future`], which will be
/// completed once the information becomes available.
///
/// This producer has an internal polling thread and as such it doesn't need to
/// be polled. It can be cheaply cloned to get a reference to the same
/// underlying producer. The internal polling thread will be terminated when the
/// `FutureProducer` goes out of scope.
#[must_use = "Producer polling thread will stop immediately if unused"]
pub struct ExporterFutureProducer<
    C = DefaultClientContext,
    R = DefaultRuntime,
    Part = NoCustomPartitioner,
> where
    Part: Partitioner,
    C: ClientContext + 'static,
{
    producer: Arc<ExporterThreadedProducer<ExporterFutureProducerContext<C>, Part>>,
    _runtime: PhantomData<R>,
}

impl<R> FromClientConfig for ExporterFutureProducer<DefaultClientContext, R>
where
    R: AsyncRuntime,
{
    fn from_config(
        config: &ClientConfig,
    ) -> KafkaResult<ExporterFutureProducer<DefaultClientContext, R>> {
        ExporterFutureProducer::from_config_and_context(config, DefaultClientContext)
    }
}

impl<C, R> FromClientConfigAndContext<C> for ExporterFutureProducer<C, R>
where
    C: ClientContext + 'static,
    R: AsyncRuntime,
{
    fn from_config_and_context(
        config: &ClientConfig,
        context: C,
    ) -> KafkaResult<ExporterFutureProducer<C, R>> {
        let future_context = ExporterFutureProducerContext {
            wrapped_context: context,
        };
        let threaded_producer =
            ExporterThreadedProducer::from_config_and_context(config, future_context)?;
        Ok(ExporterFutureProducer {
            producer: Arc::new(threaded_producer),
            _runtime: PhantomData,
        })
    }
}

impl<C, R> ExporterFutureProducer<C, R>
where
    C: ClientContext + 'static,
    R: AsyncRuntime,
{
    /// Sends a message to Kafka, returning the result of the send.
    ///
    /// The `queue_timeout` parameter controls how long to retry for if the
    /// librdkafka producer queue is full. Set it to `Timeout::Never` to retry
    /// forever or `Timeout::After(0)` to never block. If the timeout is reached
    /// and the queue is still full, an [`RDKafkaErrorCode::QueueFull`] error will
    /// be reported in the [`OwnedDeliveryResult`].
    ///
    /// Keep in mind that `queue_timeout` only applies to the first phase of the
    /// send operation. Once the message is queued, the underlying librdkafka
    /// client has separate timeout parameters that apply, like
    /// `delivery.timeout.ms`.
    ///
    /// See also the [`FutureProducer::send_result`] method, which will not
    /// retry the queue operation if the queue is full.
    pub async fn send<K, P, T>(
        &self,
        record: ExporterFutureRecord<'_, K, P>,
        queue_timeout: T,
    ) -> OwnedDeliveryResult
    where
        K: ToBytes + ?Sized,
        P: ToBytes + ?Sized,
        T: Into<Timeout>,
    {
        let start_time = Instant::now();
        let queue_timeout = queue_timeout.into();
        let can_retry = || match queue_timeout {
            Timeout::Never => true,
            Timeout::After(t) if start_time.elapsed() < t => true,
            _ => false,
        };

        let (tx, rx) = oneshot::channel();
        let mut base_record = record.into_base_record(Box::new(tx));

        loop {
            match self.producer.send(base_record) {
                Err((e, record))
                    if e == KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull)
                        && can_retry() =>
                {
                    base_record = record;
                    R::delay_for(Duration::from_millis(100)).await;
                }
                Ok(_) => {
                    // We hold a reference to the producer, so it should not be
                    // possible for the producer to vanish and cancel the
                    // oneshot.
                    break rx.await.expect("producer unexpectedly dropped");
                }
                Err((e, record)) => {
                    let owned_message = OwnedMessage::new(
                        record.payload.map(|p| p.to_bytes().to_vec()),
                        record.key.map(|k| k.to_bytes().to_vec()),
                        record.topic.to_owned(),
                        record
                            .timestamp
                            .map_or(Timestamp::NotAvailable, Timestamp::CreateTime),
                        record.partition.unwrap_or(-1),
                        0,
                        record.headers,
                    );
                    break Err((e, owned_message));
                }
            }
        }
    }

    /// Like [`FutureProducer::send`], but if enqueuing fails, an error will be
    /// returned immediately, alongside the [`ExporterFutureRecord`] provided.
    #[allow(clippy::result_large_err)]
    #[allow(dead_code)]
    pub fn send_result<'a, K, P>(
        &self,
        record: ExporterFutureRecord<'a, K, P>,
    ) -> Result<ExporterDeliveryFuture, (KafkaError, ExporterFutureRecord<'a, K, P>)>
    where
        K: ToBytes + ?Sized,
        P: ToBytes + ?Sized,
    {
        let (tx, rx) = oneshot::channel();
        let base_record = record.into_base_record(Box::new(tx));
        self.producer
            .send(base_record)
            .map(|()| ExporterDeliveryFuture { rx })
            .map_err(|(e, record)| (e, ExporterFutureRecord::from_base_record(record)))
    }

    /// Polls the internal producer.
    ///
    /// This is not normally required since the `FutureProducer` has a thread
    /// dedicated to calling `poll` regularly.
    #[allow(dead_code)]
    pub fn poll<T: Into<Timeout>>(&self, timeout: T) {
        self.producer.poll(timeout);
    }
}

impl<C, R, Part> Producer<ExporterFutureProducerContext<C>, Part>
    for ExporterFutureProducer<C, R, Part>
where
    C: ClientContext + 'static,
    R: AsyncRuntime,
    Part: Partitioner,
{
    fn client(&self) -> &Client<ExporterFutureProducerContext<C>> {
        self.producer.client()
    }

    fn flush<T: Into<Timeout>>(&self, timeout: T) -> KafkaResult<()> {
        self.producer.flush(timeout)
    }

    fn purge(&self, flags: PurgeConfig) {
        self.producer.purge(flags)
    }

    fn in_flight_count(&self) -> i32 {
        self.producer.in_flight_count()
    }

    fn init_transactions<T: Into<Timeout>>(&self, timeout: T) -> KafkaResult<()> {
        self.producer.init_transactions(timeout)
    }

    fn begin_transaction(&self) -> KafkaResult<()> {
        self.producer.begin_transaction()
    }

    fn send_offsets_to_transaction<T: Into<Timeout>>(
        &self,
        offsets: &TopicPartitionList,
        cgm: &ConsumerGroupMetadata,
        timeout: T,
    ) -> KafkaResult<()> {
        self.producer
            .send_offsets_to_transaction(offsets, cgm, timeout)
    }

    fn commit_transaction<T: Into<Timeout>>(&self, timeout: T) -> KafkaResult<()> {
        self.producer.commit_transaction(timeout)
    }

    fn abort_transaction<T: Into<Timeout>>(&self, timeout: T) -> KafkaResult<()> {
        self.producer.abort_transaction(timeout)
    }
}

/// The [`ExporterFutureProducerContext`] used by the [`ExporterFutureProducer`].
///
/// This context will use a [`Future`] as its `DeliveryOpaque` and will complete
/// the future when the message is delivered (or failed to).
#[derive(Clone)]
pub struct ExporterFutureProducerContext<C: ClientContext + 'static> {
    wrapped_context: C,
}

// Delegates all the methods calls to the wrapped context.
impl<C: ClientContext + 'static> ClientContext for ExporterFutureProducerContext<C> {
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = C::ENABLE_REFRESH_OAUTH_TOKEN;

    fn log(&self, level: RDKafkaLogLevel, fac: &str, log_message: &str) {
        self.wrapped_context.log(level, fac, log_message);
    }

    fn stats(&self, statistics: Statistics) {
        self.wrapped_context.stats(statistics);
    }

    fn stats_raw(&self, statistics: &[u8]) {
        self.wrapped_context.stats_raw(statistics)
    }

    fn error(&self, error: KafkaError, reason: &str) {
        self.wrapped_context.error(error, reason);
    }

    fn generate_oauth_token(
        &self,
        oauthbearer_config: Option<&str>,
    ) -> Result<OAuthToken, Box<dyn Error>> {
        self.wrapped_context
            .generate_oauth_token(oauthbearer_config)
    }
}

impl<C, Part> ProducerContext<Part> for ExporterFutureProducerContext<C>
where
    C: ClientContext + 'static,
    Part: Partitioner,
{
    type DeliveryOpaque = Box<oneshot::Sender<OwnedDeliveryResult>>;

    fn delivery(
        &self,
        delivery_result: &DeliveryResult<'_>,
        tx: Box<oneshot::Sender<OwnedDeliveryResult>>,
    ) {
        let owned_delivery_result = match *delivery_result {
            Ok(ref message) => Ok(Delivery {
                partition: message.partition(),
                offset: message.offset(),
                timestamp: message.timestamp(),
            }),
            Err((ref error, ref message)) => Err((error.clone(), message.detach())),
        };
        let _ = tx.send(owned_delivery_result); // TODO: handle error
    }
}

/// A [`Future`] wrapping the result of the message production.
///
/// Once completed, the future will contain an `OwnedDeliveryResult` with
/// information on the delivery status of the message. If the producer is
/// dropped before the delivery status is received, the future will instead
/// resolve with [`oneshot::Canceled`].
pub struct ExporterDeliveryFuture {
    rx: oneshot::Receiver<OwnedDeliveryResult>,
}

impl Future for ExporterDeliveryFuture {
    type Output = Result<OwnedDeliveryResult, oneshot::Canceled>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.rx.poll_unpin(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdkafka::message::OwnedHeaders;

    // ---- ExporterFutureRecord builder ----

    #[test]
    fn future_record_to_creates_record_with_topic() {
        let record = ExporterFutureRecord::<str, [u8]>::to("my-topic");
        assert_eq!(record.topic, "my-topic");
        assert!(record.partition.is_none());
        assert!(record.payload.is_none());
        assert!(record.key.is_none());
        assert!(record.timestamp.is_none());
        assert!(record.headers.is_none());
    }

    #[test]
    fn future_record_payload_sets_payload() {
        let data = b"hello world";
        let record = ExporterFutureRecord::<str, [u8]>::to("t").payload(data);
        assert_eq!(record.payload, Some(data.as_slice()));
    }

    #[test]
    fn future_record_key_sets_key() {
        let record = ExporterFutureRecord::<str, [u8]>::to("t").key("my-key");
        assert_eq!(record.key, Some("my-key"));
    }

    #[test]
    fn future_record_partition_sets_partition() {
        let record = ExporterFutureRecord::<str, [u8]>::to("t").partition(3);
        assert_eq!(record.partition, Some(3));
    }

    #[test]
    fn future_record_timestamp_sets_timestamp() {
        let record = ExporterFutureRecord::<str, [u8]>::to("t").timestamp(1234567890);
        assert_eq!(record.timestamp, Some(1234567890));
    }

    #[test]
    fn future_record_headers_sets_headers() {
        let headers = OwnedHeaders::new().insert(rdkafka::message::Header {
            key: "format",
            value: Some(b"otlp"),
        });
        let record = ExporterFutureRecord::<str, [u8]>::to("t").headers(headers);
        assert!(record.headers.is_some());
    }

    #[test]
    fn future_record_builder_chain() {
        let data = b"payload-bytes";
        let headers = OwnedHeaders::new().insert(rdkafka::message::Header {
            key: "h1",
            value: Some(b"v1"),
        });
        let record = ExporterFutureRecord::<str, [u8]>::to("topic")
            .key("k")
            .payload(data)
            .partition(2)
            .timestamp(999)
            .headers(headers);

        assert_eq!(record.topic, "topic");
        assert_eq!(record.key, Some("k"));
        assert_eq!(record.payload, Some(data.as_slice()));
        assert_eq!(record.partition, Some(2));
        assert_eq!(record.timestamp, Some(999));
        assert!(record.headers.is_some());
    }

    #[test]
    fn future_record_into_base_record_preserves_fields() {
        let data = b"bytes";
        let record = ExporterFutureRecord::<str, [u8]>::to("topic")
            .key("key")
            .payload(data)
            .partition(1)
            .timestamp(42);

        let base = record.into_base_record(());
        assert_eq!(base.topic, "topic");
        assert_eq!(base.key, Some("key"));
        assert_eq!(base.payload, Some(data.as_slice()));
        assert_eq!(base.partition, Some(1));
        assert_eq!(base.timestamp, Some(42));
    }

    #[test]
    fn future_record_from_base_record_roundtrip() {
        let data = b"bytes";
        let original = ExporterFutureRecord::<str, [u8]>::to("topic")
            .key("key")
            .payload(data)
            .partition(5)
            .timestamp(100);

        let base = original.into_base_record(());
        let restored = ExporterFutureRecord::from_base_record(base);

        assert_eq!(restored.topic, "topic");
        assert_eq!(restored.key, Some("key"));
        assert_eq!(restored.payload, Some(data.as_slice()));
        assert_eq!(restored.partition, Some(5));
        assert_eq!(restored.timestamp, Some(100));
    }
}
