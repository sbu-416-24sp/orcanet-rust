use std::collections::HashMap;
use rand::Rng;

#[derive(Clone)]
pub struct Consumers {
    consumers: HashMap<String, Consumer>,
}

#[derive(Clone)]
pub struct ConsumerRequest {
    pub chunks_sent: u64,
    pub access_token: String,
}

#[derive(Clone)]
pub struct Consumer {
    pub wallet_address: String,
    pub requests: HashMap<String, ConsumerRequest>,
}

impl Consumers {
    pub fn new() -> Self {
        Consumers {
            consumers: HashMap::new(),
        }
    }

    pub fn add_consumer(&mut self, consumer_address: String, consumer: Consumer) {
        self.consumers.insert(consumer_address, consumer);
    }

    pub fn get_consumer(&mut self, consumer_address: &str) -> Option<&mut Consumer> {
        self.consumers.get_mut(consumer_address)
    }
}

pub fn generate_access_token() -> String {
    // Generate a completely random access token
    let mut rng = rand::thread_rng();
    let token: String = (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();

    token
}