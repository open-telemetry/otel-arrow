// A literal URN whose category segment disagrees with `category` is an error.
use otap_df_engine_macros::component_inventory;

#[component_inventory(id = "urn:otel:exporter:otlp", category = Receiver)]
pub struct Foo;

fn main() {}
