pub mod cursor;

use std::path::Path;
use std::sync::Arc;

use crate::error::BrokerResult;
use crate::log::segment::SegmentConfig;
use crate::log::LogManager;
use crate::network::server::BrokerServer;

use self::cursor::CursorManager;

pub struct LogBroker {
    pub log_manager: Arc<LogManager>,
    pub cursor_manager: Arc<CursorManager>,
}

impl LogBroker {
    pub fn new(data_dir: &Path, segment_config: SegmentConfig) -> BrokerResult<Self> {
        let log_manager = Arc::new(LogManager::new(data_dir, segment_config));
        let cursor_manager = Arc::new(CursorManager::new(data_dir)?);

        Ok(Self {
            log_manager,
            cursor_manager,
        })
    }

    pub fn start(&self, bind_addr: &str) -> BrokerResult<()> {
        let mut server = BrokerServer::new(Arc::clone(&self.log_manager), bind_addr)?;
        eprintln!("[broker] listening on {}", bind_addr);
        server.run()
    }
}
