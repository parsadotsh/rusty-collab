use std::time::Duration;

use tokio::time::sleep;

use crate::{App, State, screen_lobby::LobbyState};

pub async fn task_leave_session(app: App) {
    app.replace_state(State::Loading);

    sleep(Duration::from_millis(150)).await;

    app.replace_state(State::Lobby(LobbyState {
        join_existing: false,
        name_input: String::new(),
    }));
}
