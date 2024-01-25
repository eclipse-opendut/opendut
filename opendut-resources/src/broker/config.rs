
#[derive(Debug)]
pub struct ResourcesBrokerConfig {
    pub message_buffer_size: usize,
}

impl Default for ResourcesBrokerConfig {
    fn default() -> Self {
        Self {
            message_buffer_size: 256,
        }
    }
}
