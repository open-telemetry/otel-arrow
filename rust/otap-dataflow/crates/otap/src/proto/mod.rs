// Disallow some rustc and clippy lints for the generated code
// (applied to all modules in this file).

#![allow(unused_results)]
#![allow(missing_docs)]
#![allow(unused_qualifications)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::enum_variant_names)]
#![allow(rustdoc::invalid_html_tags)]

#[path = ""]
pub mod opentelemetry {
    #[path = ""]
    pub mod experimental {
        #[path = ""]
        pub mod arrow {
            #[path = "opentelemetry.proto.experimental.arrow.v1.rs"]
            pub mod v1;
        }
    }
}
