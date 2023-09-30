use crate::{app_state::AppState, request_handler};
use actix_web::web::Data;
use futures::{channel::mpsc::channel, SinkExt, StreamExt};
use log::{info, warn};
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};
use std::{path::Path, time::Duration};

/// Asynchronously watches for file changes and updates the application state accordingly.
///
/// This function sets up a file watcher that monitors the specified path for changes (create, modify, or remove events).
/// When a significant event occurs, it updates the application state with the new configuration.
///
/// # Arguments
///
/// * `path` - A path to the directory or file to be watched.
/// * `app_state` - A reference to the application state (`AppState`) shared across the application.
///
/// # Example
///
/// ```rust
/// use crate::AppState;
/// use actix_web::web::Data;
/// use std::path::Path;
///
/// #[actix_rt::main]
/// async fn main() {
///     let app_state = Data::new(AppState::default()); // Create application state
///
///     // Start watching a directory for changes
///     file_watcher(Path::new("./config"), app_state).await;
/// }
/// ```
///
/// # Panics
///
/// This function may panic if it encounters errors while setting up the file watcher or processing events.
/// It is advisable to handle errors appropriately in production code.
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
        .cache()
        .add_root(path.as_ref(), RecursiveMode::Recursive);

    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .expect("Failed to watch path");

    while let Some(res) = rx.next().await {
        match res {
            Ok(events) => {
                let has_significant_event = events.iter().any(|event| {
                    matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    )
                });

                if has_significant_event {
                    info!(target: "file_watcher", "File changed: {:?}", events);
                    let search_path = path.as_ref().to_string_lossy().into_owned();
                    let request_map = request_handler::create_route_map(Some(search_path));
                    let mut config_map = app_state.config_map.lock().unwrap();
                    config_map.extend(request_map);
                }
            }
            Err(e) => warn!("File watcher error: {:?}", e),
        }
    }
}
