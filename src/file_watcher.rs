use actix_web::web::Data;
use futures::{channel::mpsc::channel, SinkExt, StreamExt};
use log::{info, warn};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

use crate::{app_state::AppState, utils};

pub async fn file_watcher<P: AsRef<Path>>(path: P, app_state: Data<AppState>) {
    let (mut tx, mut rx) = channel(32);
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            //let rt = tokio::runtime::Runtime::new().unwrap();
            //rt.block_on(async {
            //    _ = tx.send(res).await;
            //})
            futures::executor::block_on(async {
                _ = tx.send(res).await;
            })
        },
        Config::default(),
    )
    .expect("Failed to create file watcher");

    watcher
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .expect("Failed to watch path");

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => match event.kind {
                EventKind::Access(_) | EventKind::Any => {}
                EventKind::Create(_)
                | EventKind::Modify(_)
                | EventKind::Remove(_)
                | EventKind::Other => {
                    info!(target: "file_watcher", "File changed: {:?}", event);
                    let search_path = path.as_ref().to_string_lossy().into_owned();
                    let request_map = utils::create_request_map(Some(search_path));
                    let mut config_map = app_state.config_map.lock().unwrap();
                    config_map.extend(request_map);
                }
            },
            Err(e) => warn!("File watcher error: {:?}", e),
        }
    }
}
