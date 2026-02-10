use core::time;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use loro::cursor::Cursor;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::App;
use crate::gossip_message::GossipMessage;
use crate::task_start_session::SessionState;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);
const CLEANUP_INTERVAL: Duration = Duration::from_millis(500);
const CACHE_TTL: Duration = Duration::from_secs(5);

pub type IdBytes = [u8; 32];
pub type AwarenessCache = HashMap<IdBytes, (Awareness, Instant)>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Awareness {
    pub endpoint_id: IdBytes,
    pub name: String,
    pub timestamp_ms: u64,
}

pub fn awareness_refresh(app: &App) -> Result<()> {
    let mut state = app.state.lock();
    let crate::State::Session(session_state) = &mut *state else {
        bail!("Expected Session state");
    };

    let timestamp_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    session_state
        .outbound_queue
        .send(GossipMessage::Awareness(Awareness {
            endpoint_id: session_state.own_id,
            name: session_state.own_name.clone(),
            timestamp_ms: timestamp_now,
        }))?;

    let instant_now = Instant::now();
    session_state
        .awareness_cache
        .retain(|_, (_, received_at)| instant_now.duration_since(*received_at) < CACHE_TTL);

    app.egui_ctx.request_repaint();

    Ok(())
}

pub fn update_awareness_cache(state: &mut SessionState, awareness: Awareness) {
    if awareness.endpoint_id == state.own_id {
        return;
    }

    let old_entry = state.awareness_cache.get(&awareness.endpoint_id);
    let should_update = if let Some((existing, _)) = old_entry {
        awareness.timestamp_ms > existing.timestamp_ms
    } else {
        true
    };

    if should_update {
        state
            .awareness_cache
            .insert(awareness.endpoint_id, (awareness, Instant::now()));
    }
}
