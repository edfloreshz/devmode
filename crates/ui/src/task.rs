//! Bridging dm-core's synchronous, blocking API into iced's async runtime.
//!
//! `dm-core` is deliberately synchronous — it's shared by the CLI, the TUI,
//! and this GUI, and none of the others want an async runtime. Rather than
//! colour the library async for one consumer, the GUI runs each blocking
//! call (SQLite queries, filesystem walks, git clones) on its own thread and
//! awaits the result, so the UI thread is never blocked and the window keeps
//! painting during long operations.

use std::future::Future;
use std::path::PathBuf;

use iced::futures::channel::oneshot;

/// Runs `f` on a dedicated thread and resolves once it returns.
///
/// Deliberately independent of whichever executor iced was built with, so
/// this keeps working regardless of the `thread-pool`/`smol`/`tokio` feature
/// the binary ends up with.
pub fn blocking<T, F>(f: F) -> impl Future<Output = T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (sender, receiver) = oneshot::channel();

    std::thread::spawn(move || {
        // An Err here just means the UI dropped the receiver (the task was
        // aborted); the work is finished either way, so there's nothing to do.
        let _ = sender.send(f());
    });

    async move {
        receiver
            .await
            .expect("dm-core worker thread panicked before sending a result")
    }
}

/// Opens the OS's native folder picker, resolving to `None` if the user
/// cancels.
///
/// `rfd`'s default features give each platform its real picker: Explorer on
/// Windows, NSOpenPanel on macOS, and — via `xdg-portal` — the desktop's own
/// xdg-desktop-portal chooser on Linux, so this respects Flatpak/sandboxed
/// environments instead of shelling out to a bundled GTK dialog.
pub async fn pick_folder(title: &'static str, starting: Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new().set_title(title);

    if let Some(starting) = starting.filter(|path| path.is_dir()) {
        dialog = dialog.set_directory(starting);
    }

    dialog
        .pick_folder()
        .await
        .map(|handle| handle.path().to_path_buf())
}
