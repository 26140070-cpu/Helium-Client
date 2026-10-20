use log::LevelFilter;
use std::path::Path;
use std::sync::Once;

const LOG_FILE: &str = "agent_loader.log";

static INIT: Once = Once::new();

pub fn init(dir: &Path) {
    INIT.call_once(|| {
        let path = dir.join(LOG_FILE);
        if let Err(e) = protocol::init_file_logger(&path, LevelFilter::Debug) {
            eprintln!("[agent_loader] file logging disabled: {e}");
        }
    });
}
