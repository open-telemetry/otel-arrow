Things that are worrying me and I wish for you to all have this information so that you can help while I'm (hopefully) in the snow.

The work stream I would be working on myself next week is listed below. I trust Utkarsh or Lalit to talk on any of these tasks.

1. Histogram of processor duration. All procsesors, we think, can be instrumented in one place. Utkarsh has proposed a very nice approach to output MMSC to Histogram instruments, very reachable.

2. Internal pipeline metrics [#2018]: this is to extend the automatic instrumentation which Laurent added, that counts data on the forward path, with new instrumentation. The goal is to reach parity with the [Collector](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md) and/or understand exactly what weee thing can/should be done instead.
   
3. Histogram for pipeline duration. After #1 and #2 this would be a detailed level metric computed by storing a new/optional timestamp in the Context.

The work I am assisting with:

1. Andres on #2026, how we want the ability to add custom attributes to the node definition. I asked for changes, more or less arriving at https://github.com/open-telemetry/otel-arrow/pull/2042 I am not sure Laurent will 100% like this, nor will Andres. I added to the node instrumentation scope using `#[compose]` a single key named `custom` with a map of user-defined key:values. I made it one constant key because the descriptor is `'static`. I'm not sure what Laurent would prefer, probably a strong schema that Weaver can validate.
2. Gokhan re: extension support. I've discussed this with Utkarsh, and I believe with help from Rust experts we can find a workable solution.  Laurent doesn't have strong opinions other than about syynchronization. Gokhan's prototypes looked good to me. Add shared and local extension wrappers, a registry, a lookup-binding interface, etc.
