use crate::engine::{Receiver, Sender, extractor::ExtractorChannelTx, scanner::ScannedDocument};

pub type DecompressorChannelTx = Sender<ScannedDocument>;
pub type DecompressorChannelRx = Receiver<ScannedDocument>;

pub struct DecompressorEngine {
    channel_tx: DecompressorChannelTx,
    channel_rx: DecompressorChannelRx,
    channel_extractor_tx: ExtractorChannelTx,
}

impl DecompressorEngine {
    pub fn new(
        channel_tx: DecompressorChannelTx,
        channel_rx: DecompressorChannelRx,
        channel_extractor_tx: ExtractorChannelTx,
    ) -> Self {
        DecompressorEngine {
            channel_tx,
            channel_rx,
            channel_extractor_tx,
        }
    }
}