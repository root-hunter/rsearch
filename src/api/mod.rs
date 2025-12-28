use crate::engine::{scanner::ScannerChannelTx};

pub struct Api {
    channel_scanner_tx: ScannerChannelTx,
}

impl Api {
    pub fn new(channel_scanner_tx: ScannerChannelTx) -> Self {
        Api { channel_scanner_tx }
    }

    pub fn scan_path(&self, path: String) -> Result<(), String> {
        self.channel_scanner_tx
            .send(path)
            .map_err(|e| format!("Failed to send scan command: {}", e))
    }
}