use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use iroh::{Endpoint, protocol::Router};
use iroh_gossip::{Gossip, TopicId, api::Event};
use loro::LoroDoc;
use tokio::{
    select,
    sync::mpsc::{self, UnboundedSender},
    task::JoinHandle,
    time::interval,
};
use tokio_stream::StreamExt;

use postcard::{from_bytes, to_allocvec as to_bytes};

use crate::{
    App, State,
    awareness::{AwarenessCache, IdBytes, awareness_refresh},
    gossip_message::{GossipMessage, handle_gossip_message},
};

pub async fn task_start_session(app: App, name: String, existing_peer: Option<String>) {
    let old_state = app.replace_state(State::Loading);

    let Ok(session_state) = setup(&app, name, existing_peer).await else {
        app.replace_state(old_state);
        return;
    };

    app.replace_state(State::Session(session_state));
}
pub struct SessionState {
    pub own_id: IdBytes,
    pub own_name: String,

    pub loro_doc: LoroDoc,
    pub loro_sub: loro::Subscription,
    pub iroh_endpoint: Endpoint,
    pub iroh_gossip: Gossip,
    pub iroh_router: Router,

    pub awareness_cache: AwarenessCache,
    pub outbound_queue: OutboundQueue,
    pub main_loop_handle: JoinHandle<Result<()>>,
}

pub type OutboundQueue = UnboundedSender<GossipMessage>;

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

    let loro_doc = LoroDoc::new();
    let loro_sub = {
        let outbound_queue = outbound_queue.clone();
        loro_doc.subscribe_local_update(Box::new(move |bytes| {
            let _ = outbound_queue.send(GossipMessage::Update {
                data: bytes.to_vec(),
            });
            true
        }))
    };

    let main_loop_handle: JoinHandle<Result<()>> = tokio::spawn({
        let mut app = app.clone();
        let outbound_queue = outbound_queue.clone();
        let loro_doc = loro_doc.clone();
        let mut awareness_interval = interval(Duration::from_millis(500));
        async move {
            loop {
                select! {
                    Some(event) = gossip_topic.next() => {
                        if let Ok(Event::Received(message)) = event {
                            let (_nonce, gossip_message): (u128, GossipMessage) = from_bytes(&message.content)?;
                            handle_gossip_message(gossip_message, &mut app, &loro_doc, &outbound_queue)?;
                        }
                    }
                    Some(message) = outbound_queue_rx.recv() => {
                        let bytes = to_bytes(&(rand::random::<u128>(), &message))?;
                        gossip_topic.broadcast(bytes.into()).await?;
                    }
                    _ = awareness_interval.tick() => {
                        awareness_refresh(&app)?;
                    }
                }
            }
        }
    });

    outbound_queue.send(GossipMessage::RequestData)?;

    Ok(SessionState {
        own_id: iroh_endpoint.id().as_bytes().to_owned(),
        own_name: name,
        loro_doc,
        loro_sub,
        iroh_endpoint,
        iroh_gossip,
        iroh_router,
        awareness_cache: HashMap::new(),
        outbound_queue: outbound_queue,
        main_loop_handle,
    })
}
