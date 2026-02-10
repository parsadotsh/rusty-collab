use std::time::Duration;

use anyhow::{Result, bail};
use iroh::{Endpoint, protocol::Router};
use iroh_gossip::{Gossip, TopicId, api::Event};
use tokio::{
    select,
    sync::mpsc::{self, UnboundedSender},
    task::JoinHandle,
};
use tokio_stream::StreamExt;
use wincode_derive::{SchemaRead, SchemaWrite};

use crate::{App, State};

pub async fn task_start_session(app: App, name: String, existing_peer: Option<String>) {
    let old_state = app.replace_state(State::Loading);

    let Ok(session_state) = setup(&app, name, existing_peer).await else {
        app.replace_state(old_state);
        return;
    };

    app.replace_state(State::Session(session_state));
}
pub struct SessionState {
    pub doc: String,
    pub iroh_endpoint: Endpoint,
    pub iroh_gossip: Gossip,
    pub iroh_router: Router,
    pub outbound_queue: OutboundQueue,
    pub main_loop_handle: JoinHandle<Result<()>>,
}

type OutboundQueue = UnboundedSender<GossipMessage>;

async fn setup(app: &App, name: String, existing_peer: Option<String>) -> Result<SessionState> {
    const GOSSIP_MAX_MESSAGE_SIZE: usize = 2 * 1024 * 1024;
    const TOPIC_ID_BYTES: [u8; 32] = [23u8; 32];

    let iroh_endpoint = Endpoint::bind().await?;
    let iroh_gossip = Gossip::builder()
        .max_message_size(GOSSIP_MAX_MESSAGE_SIZE)
        .spawn(iroh_endpoint.clone());
    let iroh_router = Router::builder(iroh_endpoint.clone())
        .accept(iroh_gossip::ALPN, iroh_gossip.clone())
        .spawn();

    let bootstrap_nodes = if let Some(existing_peer) = &existing_peer {
        vec![existing_peer.parse()?]
    } else {
        vec![]
    };

    let mut gossip_topic = iroh_gossip
        .subscribe(TopicId::from_bytes(TOPIC_ID_BYTES), bootstrap_nodes)
        .await?;

    if existing_peer.is_some() {
        gossip_topic.joined().await?;
    }

    let (outbound_queue, mut outbound_queue_rx) = mpsc::unbounded_channel::<GossipMessage>();

    let main_loop_handle: JoinHandle<Result<()>> = tokio::spawn({
        let mut app = app.clone();
        let outbound_queue = outbound_queue.clone();
        async move {
            loop {
                select! {
                    Some(event) = gossip_topic.next() => {
                        if let Ok(Event::Received(message)) = event {
                            let (_nonce, gossip_message): (u128, GossipMessage) = wincode::deserialize(&message.content)?;
                            handle_gossip_message(gossip_message, &mut app, &outbound_queue)?;
                        }
                    }
                    Some(message) = outbound_queue_rx.recv() => {
                        let bytes = wincode::serialize(&(rand::random::<u128>(), &message))?;
                        gossip_topic.broadcast(bytes.into()).await?;
                    }
                }
            }
        }
    });

    outbound_queue.send(GossipMessage::RequestData)?;

    Ok(SessionState {
        doc: String::new(),
        iroh_endpoint,
        iroh_gossip,
        iroh_router,
        outbound_queue: outbound_queue,
        main_loop_handle,
    })
}

#[derive(SchemaRead, SchemaWrite)]
pub enum GossipMessage {
    RequestData,
    Update { new_doc: String },
}

fn handle_gossip_message(
    message: GossipMessage,
    app: &mut App,
    outbound_queue: &OutboundQueue,
) -> Result<()> {
    let mut state = app.state.lock();
    let State::Session(session_state) = &mut *state else {
        bail!("Expected Session state");
    };

    match message {
        GossipMessage::RequestData => {
            outbound_queue.send(GossipMessage::Update {
                new_doc: session_state.doc.clone(),
            })?;
        }
        GossipMessage::Update { new_doc } => {
            session_state.doc = new_doc;
            app.egui_ctx.request_repaint();
        }
    }

    Ok(())
}
