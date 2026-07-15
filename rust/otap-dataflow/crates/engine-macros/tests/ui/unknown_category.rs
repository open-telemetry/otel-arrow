// A misspelled category must be a compile error, not a silent bad entry.
use otap_df_engine_macros::component_inventory;

#[component_inventory(category = Reciever)]
pub struct Foo;

fn main() {}
