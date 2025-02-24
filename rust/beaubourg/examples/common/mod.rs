/// Just a basic message.
///
/// Note: a message must be at the minimum 'static + Clone + Send.
#[derive(Clone, Debug)]
pub struct Message {
    pub origin: String,
    #[allow(dead_code)]
    pub payload: usize,
}
