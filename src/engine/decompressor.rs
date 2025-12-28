use crate::engine::{Receiver, Sender, scanner::ScannedDocument};

pub struct DecompressorEngine {
    channel_tx: Sender<ScannedDocument>,
    channel_rx: Receiver<ScannedDocument>,
    channel_extractor_tx: Sender<ScannedDocument>,
}

impl DecompressorEngine {
    pub fn new(
        channel_tx: Sender<ScannedDocument>,
        channel_rx: Receiver<ScannedDocument>,
        channel_extractor_tx: Sender<ScannedDocument>,
    ) -> Self {
        DecompressorEngine {
            channel_tx,
            channel_rx,
            channel_extractor_tx,
        }
    }
}
