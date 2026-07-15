// A non-factory item (no `name` field) must supply an explicit `id`.
use otap_df_engine_macros::component_inventory;

#[component_inventory(category = Extension)]
pub struct AdminServer;

fn main() {}
