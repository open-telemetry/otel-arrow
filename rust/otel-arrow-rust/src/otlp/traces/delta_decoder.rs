use std::ops::Add;

#[derive(Default)]
pub struct DeltaDecoder<K, V> {
    prev_key_value: Option<(K, V)>,
}

impl<K, V> DeltaDecoder<K, V>
where
    K: Eq + Clone,
    V: Add<Output = V> + Copy,
{
    pub(crate) fn new() -> Self {
        Self {
            prev_key_value: None,
        }
    }
    pub(crate) fn decode(&mut self, key: &K, value: V) -> V {
        let Some((prev_key, prev_value)) = std::mem::take(&mut self.prev_key_value) else {
            self.prev_key_value = Some((key.clone(), value));
            return value;
        };

        if &prev_key == key {
            let value = prev_value.add(value);
            self.prev_key_value = Some((prev_key, value));
            value
        } else {
            // new key, reset value.
            self.prev_key_value = Some((key.clone(), value));
            value
        }
    }
}
