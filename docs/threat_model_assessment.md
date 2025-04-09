# Threat Model Assessment

## Scope

This threat model is scoped to the proposed changes in the application layer
message structure and compression functionality. As such, the gRPC endpoint will
inherit the security of existing infrastructure. Threats related to general
implementation, authentication of systems, transport layer security, and data
storage are out of scope for this threat model. The existing OTEL guidance is
recommended to mitigate threats in these respective areas.

* General: [security best
  practices](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/security-best-practices.md)
* Transport Layer Security: [setting up
  certificates](https://opentelemetry.io/docs/collector/configuration/#setting-up-certificates)
* Authentication: [custom
  auth](https://opentelemetry.io/docs/collector/custom-auth/)

![diagram flow and scope](img/OTEL%20-%20TMA%20scope.png)

## Security Testing

For this project we intend to run fuzz tests and various other security scans on
each commit.

We will remediate and disclose the findings as the stakeholders see fit. If you
privately discover a vulnerability in the implementation please follow our
process for reporting a security vulnerability.

* [ToDo] List the fuzz testing tools, source code scanners here (should be
  transparent about what is run and what we do with those findings)
* [ToDo] List of public results for fuzz tests and security scanners here (if
  you want to share them openly or keep them a secret is up to you. usually an
  embargo of vulns is desirable but that is your call)

## Threat Model

> Note: The following threats in the STRIDE model are not considered in this
assessment because we consider them to be the responsibility of the security
layer of the OTEL collector itself: spoofing identity, tampering with data,
repudiation.

### STRIDE: Denial of Service

Asset: Availability of system resources on collector(s).

Threat: Attacker can send some form of maliciously compressed data, resulting in
DoS conditions similar to those seen in "zip bombs".

Mitigation:

1) This implementation is currently using the ZSTD library
   (github.com/klauspost/compress/zstd). TODO: See if those are problematic
   using this compression algo. If so, a sample mitigation can be to avoid
   unpacking these types of bombs (if we can detect them), having a max
   decompression time or resource allocation size limit (configurable), and
   probably some others I can't think of. ZSTD seems immune to "zip bomb" issue
   (need to be double checked, see
   https://github.com/klauspost/compress/discussions/727#discussioncomment-4590889)
2) Fuzz testing can help discover these types of bugs sooner rather than later.

### STRIDE: Denial of Service (cardinality)

Asset: Availability of system resources on collector(s).

Threat: Attacker can send Arrow dictionary messages with a large number of
unique values, resulting in DoS conditions involving very high memory
consumption.

Mitigation:

1) Arrow schemas specifying dictionaries with an index size greater than 2^16
(or 2^32?) are systematically considered invalid. The corresponding OTLP Arrow
connection will be closed with an error message.
2) The protocol also provides for automatic switching to a dictionary-free
version of columns that exceed the previously specified cardinality limit. SDK
clients must implement this mechanism or they could be detected as malicious
producers. [ToDo check implementation]

### STRIDE: Denial of Service (connection count)

Asset: Availability of system resources on collector(s).

Threat: Attacker can open many OTLP Arrow connections, resulting in DoS
conditions involving very high memory consumption as the OTLP Arrow protocol is
stateful.

Mitigation: Two protections are in place to mitigate this threat: a maximum
number of simultaneous connections per collector and a maximum connection time.
The first protection limits the number of resources used by the collector
(memory, network connection) and the second protection avoids having unused
connections and dictionary entries. [ToDo check implementation]

### STRIDE: Information disclosure

Asset: Data confidentiality.

Threat: [ToDo] flawed error responses.

### [ToDo] More to come

* invalid, or compromised inputs causing security or reliability issues.
* very large input data causing denial of service.
* high cardinality data causing dictionary overflow (over multiple messages).
* ... TBD

Check this issue for complementary information:
https://github.com/open-telemetry/opentelemetry-specification/issues/1891

## Acknowledgements

Special thanks to Jordan Zebor (F5) for his contribution to the threat model
assessment.
