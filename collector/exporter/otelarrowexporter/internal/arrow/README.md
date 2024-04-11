# OTel-Arrow streaming exporter

## Design

The principle components of the OTel-Arrow Exporter are:

1. Sender logic: Submits data to a stream, waits for response or timeout.
2. Prioritizer logic: Intermediary, configurable policy for choice of stream
3. Manager logic: Oversees multiple stream lifetimes, decides to downgrade
4. Stream logic: A single gRPC OTAP stream, consisting of independent
   reader and writer subroutines.

A request goes through the following steps following arrival inside
this component.

The sender computes per-request metadata including auth headers and
the original uncompressed data size, while still in the caller's
context.  Then, it checks with the prioritizer for the downgrade
condition, otherwise submits the item to a stream via the prioritizer.

The prioritizer dictates which current stream receives arriving items
of data to balance load across streams.  The prioritizer
implementations are described in a section below.

The stream manager is responsible for supervising individual streams
and outcomes.  The stream manager is responsible for the decision to
downgrade to standard OTLP when it appears that Arrow is unsupported.

The individual stream is made up of two subroutines, _reader_ and
_writer_, executing independently and a "waiters" map.

The stream writer receives work from the sender logic (via the
prioritizer), encodes the data using the current OTel-Arrow stream
state, and writes it via gRPC.  As soon as the data is entered into
the gRPC write buffer, the stream writer continues to the next
request.  This repeats until the stream maximum lifetime is reached.

The stream reader receives status data from the corresponding
OTel-Arrow receiver, translates the responses into `error` objects,
and returns them to the awaiting sender logic.

### Context hierarchy

There are three levels of Context object used inside the Exporter in
addition to the Context object that arrives from the pipeline when
data is sent.
