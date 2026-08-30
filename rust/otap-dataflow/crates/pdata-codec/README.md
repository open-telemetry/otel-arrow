# PData codecs

This crate owns the extension boundary between independently decodable byte
formats and native OTAP Arrow records. Codecs register immutable factories at
link time. Each pipeline runtime validates that registry once, then creates
mutable decoder and encoder instances lazily.

A decode-only extension implements `PdataDecoder` and registers a decoder
factory. It does not need an encoder, a core enum variant, or engine changes.
Payload and node integration live in later layers so this crate does not depend
on the pipeline engine or delivery context.
