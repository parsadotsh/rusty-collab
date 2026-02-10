use anyhow::Result;
use anyhow::bail;
use loro::LoroDoc;
use serde_derive::{Deserialize, Serialize};

use crate::State;
use crate::awareness;
use crate::awareness::Awareness;
use crate::{App, task_start_session::OutboundQueue};

#[derive(Serialize, Deserialize)]
pub enum GossipMessage {
    RequestData,
    Update { data: Vec<u8> },
    Awareness(Awareness),
}

pub fn handle_gossip_message(
    message: GossipMessage,
    app: &mut App,
    loro_doc: &LoroDoc,
    outbound_queue: &OutboundQueue,
) -> Result<()> {
    let mut state = app.state.lock();
    let State::Session(session_state) = &mut *state else {
        bail!("Expected Session state");
    };

    match message {
        GossipMessage::RequestData => {
            let snapshot = loro_doc.export(loro::ExportMode::Snapshot)?;
            let _ = outbound_queue.send(GossipMessage::Update { data: snapshot });
        }
        GossipMessage::Update { data } => {
            loro_doc.import(&data)?;
            session_state.egui_cursors_needs_update = true;
            app.egui_ctx.request_repaint();
        }
        GossipMessage::Awareness(awareness) => {
            awareness::update_awareness_cache(session_state, awareness);
            app.egui_ctx.request_repaint();
        }
    }

    Ok(())
}
