use crate::{app_state::AppState, request_handler};
use actix_web::web::Data;
use futures::{channel::mpsc::channel, SinkExt, StreamExt};
use log::{info, warn};
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};
use std::{path::Path, time::Duration};

pub async fn file_watcher<P: AsRef<Path>>(path: P, app_state: Data<AppState>) {
    let (mut tx, mut rx) = channel(32);

    let mut debouncer = new_debouncer(
        Duration::from_secs(2),
        None,
        move |res: DebounceEventResult| {
            futures::executor::block_on(async {
                _ = tx.send(res).await;
            })
        },
    )
    .unwrap();

    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .expect("Failed to watch path");

    debouncer
        .cache()
        .add_root(path.as_ref(), RecursiveMode::Recursive);

    while let Some(res) = rx.next().await {
        match res {
            Ok(events) => events.iter().for_each(|event| match event.kind {
                EventKind::Access(_) | EventKind::Any => {}
                EventKind::Create(_)
                | EventKind::Modify(_)
                | EventKind::Remove(_)
                | EventKind::Other => {
                    info!(target: "file_watcher", "File changed: {:?}", event);
                    let search_path = path.as_ref().to_string_lossy().into_owned();
                    let request_map = request_handler::create_request_map(Some(search_path));
                    let mut config_map = app_state.config_map.lock().unwrap();
                    config_map.extend(request_map);
                }
            }),
            Err(e) => warn!("File watcher error: {:?}", e),
        }
    }
}
