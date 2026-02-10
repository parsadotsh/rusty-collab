use crate::{App, State, screen_lobby::LobbyState};

pub async fn task_leave_session(app: App) {
    let old_state = app.replace_state(State::Loading);

    if let State::Session(session_state) = old_state {
        let _ = session_state.iroh_gossip.shutdown().await;
        session_state.main_loop_handle.abort();
        let _ = session_state.iroh_router.shutdown().await;
        session_state.iroh_endpoint.close().await;
    }

    app.replace_state(State::Lobby(LobbyState {
        join_existing: false,
        name_input: String::new(),
        existing_peer_input: String::new(),
    }));
}
