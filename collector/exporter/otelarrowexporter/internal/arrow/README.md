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

![Block diagram of OpenTelemetry Arrow Exporter](./design.png)

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

There are three levels of Context object used inside the Exporter, in
addition to the Context object that arrives from the pipeline when
data is sent.

The top-level Context corresponds with the component lifetime.  This
context is canceled when the component is shutdown.

As a child of the top-level Context, the downgrade Context is one that
will be canceled when the stream manager decides to downgrade.

As a child of the downgrade Context, the individual Stream context is
one that will be canceled when the stream itself ends or closes in any
way.

In request context, the Exporter sender logic arrives with a timeout
that is configured in the `exporterhelper`, via a `timeout` field in
the configuration.

### Stream lifecycle

When a stream starts, it first calculates a per-stream "jitter" factor
which is subtracted from the configured maximum stream lifetime.  The
stream will send new requests until the timer expires, and then it
will `CloseSend()` the gRPC stream, which signals to the receiver to
initiate stream shutdown.  When the receiver has sent its last
response, it returns and the exporter sees end-of-file.

When either the reader or the writer see an error condition, they will
cancel the stream and cause the other subroutine to return.  Both
errors are logged, and the stream will be restarted.

The stream begins with a `*streamWorkState`, a reference to a
structure including the stream `input` (`chan<- writeItem`) and
`waiters` (`map[BatchID]<-chan error`).  The `*streamWorkState` is
re-used across streams, passed by the stream manager from the exiting
stream to the new stream to use when it starts.

In situations where a stream breaks while some work is in flight, the
special `ErrStreamRestarting` error code is returned to indicate that
a stream broke, a condition not to the data.  This causes the sender
logic to immediately restart the operation on a new stream, instead of
returning a retryable error code to the `exporterhelper` logic, which
would delay before retrying.

### Downgrade

The downgrade mechanism is implemented by canceling the special
Context that was created for this purpose.  Once downgrade has
happened, the prioritizer is expected to return a `nil` Stream and
`nil` error value.  When the sender logic sees this condition, it
returns to the standard OTLP Exporter that called in to this package.

Synchronization around the downgrade is relatively simple, however it
is required to leave behind one or more goroutines in the background,
in case of races between the prioritizer and sender logic.  There is a
method named `drain()` that will reply with `ErrStreamRestarting` to
any `writeItem` values that arrive after downgrade happens.

TODO: Fix https://github.com/open-telemetry/otel-arrow/issues/87.
Note that re-use of `*streamWorkState` across restart may lead to
abandoned work when some streams are unavailable, because no streams
restart following unavaiable, instead the manager waits for downgrade.

### Prioritizers

#### FIFO

This prioritizer gives work to the first stream that is ready for it.
The implementation shares one `chan writeItem` between multiple
`streamWorkState` objects.

#### LeastLoadedN

This prioritizer randomly selects N active streams and chooses the one
with the least outstanding number of items of data.  This is
accomplished using a number of intermediary subroutines, which
repeatly get a next item of data, pick a stream, and place the item
into the stream's input channel.  To select a least-loaded prioritizer,
use "leastloaded" followed by N (e.g., "leastloaded2", "leastloaded10").

TODO: Note that this prioritizer does not consider immediate readiness
of the corresponding stream, making it possible for intermediate
prioritizer subroutines to block when there is more than one pending
work item for a given stream.  With numStreams intermediate
subroutines available, there is a chance of blocking when in fact a
more-loaded stream is ready.  Consider a change to select from only
the immediately ready streams, although recognize that it will take 
a special case when no streams are immediately ready.
