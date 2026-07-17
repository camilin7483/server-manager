use sm_core::id::ServerId;
use sm_core::types::ConnectionInfo;
use sm_net::{SessionManager, SshOutput};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

type PendingResult<T> = Arc<Mutex<Option<Result<T, String>>>>;

pub struct ConnectionBridge {
    runtime: Runtime,
    manager: Arc<SessionManager>,
    pending: Arc<Mutex<HashMap<ServerId, Vec<ConnectionEvent>>>>,
}

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected(ServerId),
    Disconnected(ServerId),
    ConnectionError(ServerId, String),
    CommandOutput(ServerId, SshOutput),
    CommandError(ServerId, String),
}

impl ConnectionBridge {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("failed to create tokio runtime");
        let manager = Arc::new(SessionManager::new(15));
        Self {
            runtime,
            manager,
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn connect(
        &self,
        server_id: ServerId,
        info: ConnectionInfo,
    ) -> Result<(), String> {
        let manager = self.manager.clone();
        let pending = self.pending.clone();
        let sid = server_id;

        self.runtime.spawn(async move {
            match manager.connect(sid, &info).await {
                Ok(()) => {
                    let mut events = pending.lock().unwrap();
                    events
                        .entry(sid)
                        .or_default()
                        .push(ConnectionEvent::Connected(sid));
                }
                Err(e) => {
                    let mut events = pending.lock().unwrap();
                    events
                        .entry(sid)
                        .or_default()
                        .push(ConnectionEvent::ConnectionError(sid, e));
                }
            }
        });

        Ok(())
    }

    pub fn disconnect(&self, server_id: ServerId) {
        let manager = self.manager.clone();
        let pending = self.pending.clone();
        let sid = server_id;

        self.runtime.spawn(async move {
            manager.disconnect(sid).await;
            let mut events = pending.lock().unwrap();
            events
                .entry(sid)
                .or_default()
                .push(ConnectionEvent::Disconnected(sid));
        });
    }

    pub fn execute(
        &self,
        server_id: ServerId,
        command: String,
    ) {
        let manager = self.manager.clone();
        let pending = self.pending.clone();
        let sid = server_id;

        self.runtime.spawn(async move {
            match manager.execute(sid, &command).await {
                Ok(output) => {
                    let mut events = pending.lock().unwrap();
                    events
                        .entry(sid)
                        .or_default()
                        .push(ConnectionEvent::CommandOutput(sid, output));
                }
                Err(e) => {
                    let mut events = pending.lock().unwrap();
                    events
                        .entry(sid)
                        .or_default()
                        .push(ConnectionEvent::CommandError(sid, e));
                }
            }
        });
    }

    pub fn poll_events(&self) -> Vec<ConnectionEvent> {
        let mut pending = self.pending.lock().unwrap();
        let mut events = Vec::new();
        for (_, evts) in pending.iter_mut() {
            events.append(evts);
            evts.clear();
        }
        events
    }

    pub fn is_connected(&self, server_id: ServerId) -> bool {
        self.runtime
            .block_on(self.manager.is_connected(server_id))
    }

    pub fn active_connections(&self) -> Vec<ServerId> {
        self.runtime.block_on(self.manager.active_sessions())
    }
}
