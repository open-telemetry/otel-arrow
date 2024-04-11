# OTel-Arrow streaming exporter

## Design

The principle components of the OTel-Arrow Exporter are:

1. Sender logic: Submits data to a stream, waits for response or timeout.
2. Prioritizer logic: Intermediary, configurable policy for choice of stream
3. Manager logic: Oversees multiple stream lifetimes, decides to downgrade
4. Stream logic: A single gRPC OTAP stream, consisting of independent
   reader and writer subroutines.

A request goes through the following steps from arrival to export.

The sender computes per-request metadata including auth headers and
the original uncompressed data size while still in the caller's
context.  Then, it checks with the prioritizer for the downgrade
condition, otherwise submits the item to a stream via the prioritizer.

The prioritizer dictates which current stream receives arriving items
of data.  The two prioritizer 
