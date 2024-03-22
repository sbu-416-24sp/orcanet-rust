#[derive(Debug, Clone)]
pub enum MarketMessage {
    RegisterFile { cid: Vec<u8>, price: u64 },
    CheckHolders { cid: Vec<u8> },
}
