;; Copyright The OpenTelemetry Authors
;; SPDX-License-Identifier: Apache-2.0

;; Implement the engine's component interfaces without a guest SDK or WASI sysroot.
(component
  (import "otel:otap-dataflow-plugin/otel-kernels@0.1.0"
    (instance $kernels (export "pdata" (type (sub resource)))))
  (alias export $kernels "pdata" (type $pdata))
  (core module $guest
    (memory (export "memory") 1)
    ;; Zero-initialized result at 16 represents Ok(()).
    (func (export "initialize") (result i32) i32.const 16)
    ;; Return Some(input), transferring ownership of the original pdata.
    (func (export "process") (param $input i32) (result i32)
      i32.const 32 i32.const 1 i32.store8
      i32.const 36 local.get $input i32.store
      i32.const 32))
  (core instance $guest (instantiate $guest))
  (type $error (record (field "message" string)))
  (func $init async (result (result (error $error)))
    (canon lift (core func $guest "initialize") (memory (core memory $guest "memory"))))
  (func $process async (param "data" (own $pdata)) (result (option (own $pdata)))
    (canon lift (core func $guest "process") (memory (core memory $guest "memory"))))
  (instance $life (export "init-error" (type $error)) (export "initialize" (func $init)))
  (instance $processor (export "pdata" (type $pdata)) (export "process" (func $process)))
  (export "otel:otap-dataflow-plugin/lifecycle@0.1.0" (instance $life))
  (export "otel:otap-dataflow-plugin/processor@0.1.0" (instance $processor)))
