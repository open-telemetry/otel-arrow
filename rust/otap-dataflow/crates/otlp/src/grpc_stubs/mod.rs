#[allow(missing_docs)]
#[path = ""]
pub mod proto {
    #[path = ""]
    pub mod collector {
        #[path = ""]
        pub mod logs {
            #[allow(unused_qualifications)]
            #[allow(unused_results)]
            #[allow(clippy::enum_variant_names)]
            #[allow(rustdoc::invalid_html_tags)]
            #[path = "opentelemetry.proto.collector.logs.v1.rs"]
            pub mod v1;
        }
        #[path = ""]
        pub mod metrics {
            #[allow(unused_qualifications)]
            #[allow(unused_results)]
            #[allow(clippy::enum_variant_names)]
            #[allow(rustdoc::invalid_html_tags)]
            #[path = "opentelemetry.proto.collector.metrics.v1.rs"]
            pub mod v1;
        }
        #[path = ""]
        pub mod trace {
            #[allow(unused_qualifications)]
            #[allow(unused_results)]
            #[allow(clippy::enum_variant_names)]
            #[allow(rustdoc::invalid_html_tags)]
            #[path = "opentelemetry.proto.collector.trace.v1.rs"]
            pub mod v1;
        }
    }

    #[path = ""]
    pub mod logs {
        #[allow(rustdoc::invalid_html_tags)]
        #[path = "opentelemetry.proto.logs.v1.rs"]
        pub mod v1;
    }

    #[path = ""]
    pub mod metrics {
        #[allow(rustdoc::invalid_html_tags)]
        #[path = "opentelemetry.proto.metrics.v1.rs"]
        pub mod v1;
    }

    #[path = ""]
    pub mod trace {
        #[allow(rustdoc::invalid_html_tags)]
        #[path = "opentelemetry.proto.trace.v1.rs"]
        pub mod v1;
    }

    #[path = ""]
    pub mod common {
        #[allow(clippy::enum_variant_names)]
        #[path = "opentelemetry.proto.common.v1.rs"]
        pub mod v1;
    }

    #[path = ""]
    pub mod resource {
        #[path = "opentelemetry.proto.resource.v1.rs"]
        pub mod v1;
    }
}
