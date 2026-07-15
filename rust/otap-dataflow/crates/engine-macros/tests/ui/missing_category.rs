// `category` is required.
use otap_df_engine_macros::component_inventory;

#[component_inventory(description = "no category here")]
pub struct Foo;

fn main() {}
