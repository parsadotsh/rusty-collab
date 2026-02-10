use std::time::Duration;

use tokio::time::sleep;

use crate::{App, State, screen_session::SessionState};

pub async fn task_start_session(app: App) {
    app.replace_state(State::Loading);

    sleep(Duration::from_millis(100)).await;

    app.replace_state(State::Session(SessionState { doc: String::new() }));
}
