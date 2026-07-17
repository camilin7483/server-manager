use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use tracing::info;

pub struct ConfigWatcher {
    rx: mpsc::Receiver<notify::Result<Event>>,
    _watcher: notify::INotifyWatcher,
}

impl ConfigWatcher {
    pub fn new(config_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::INotifyWatcher::new(tx, notify::Config::default())?;

        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;

        info!("Observando cambios en {}", config_path.display());

        Ok(Self {
            rx,
            _watcher: watcher,
        })
    }

    pub fn poll_changes(&self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            if let Ok(event) = event {
                if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_)
                ) {
                    events.push(event);
                }
            }
        }
        events
    }
}
