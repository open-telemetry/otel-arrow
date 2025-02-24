use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub(crate) struct SingletonManager<Msg> {
    // processor_senders: Arc<Mutex<HashMap<String, flume::Sender<Vec<Msg>>>>>,
    exporter_senders: Arc<Mutex<HashMap<String, flume::Sender<Vec<Msg>>>>>,
}

impl<Msg> Default for SingletonManager<Msg>
where
    Msg: 'static + Send + Clone,
{
    fn default() -> Self {
        SingletonManager {
            // processor_senders: Arc::new(Mutex::new(HashMap::new())),
            exporter_senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<Msg> SingletonManager<Msg>
where
    Msg: 'static + Send + Clone,
{
    // pub fn set_processor_sender(&self, name: String, sender:
    // flume::Sender<Vec<Msg>>) {     let mut processor_senders =
    // self.processor_senders.lock().expect("lock processor_senders failed");
    //     if processor_senders.insert(name.clone(), sender).is_some() {
    //         tracing::warn!("processor sender {} already exists", name);
    //     }
    // }
    //
    // pub fn get_processor_sender(&self, name: &str) ->
    // Option<flume::Sender<Vec<Msg>>> {     let processor_senders =
    // self.processor_senders.lock().expect("lock processor_senders failed");
    //     processor_senders.get(name).map(|sender| sender.clone())
    // }

    pub(crate) fn get_or_create_exporter_channel(
        &self,
        name: &str,
        channel_size: usize,
    ) -> (flume::Sender<Vec<Msg>>, Option<flume::Receiver<Vec<Msg>>>) {
        let mut exporter_senders = self.exporter_senders.lock().expect("lock exporter_senders failed");
        let sender = exporter_senders.get(name).cloned();
        match sender {
            Some(sender) => (sender, None /* receiver already returned in a previous call */),
            None => {
                let (sender, receiver) = flume::bounded(channel_size);
                if exporter_senders.insert(name.to_string(), sender.clone()).is_some() {
                    panic!("exporter sender {} already exists", name);
                }
                (sender, Some(receiver))
            }
        }
    }
}
