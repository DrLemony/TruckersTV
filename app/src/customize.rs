//! The folder of the player's own videos, for the cab's TV.

use tauri::WebviewWindow;
use westplay_setup::pick;

/// Asks for the folder the cab's TV plays PC videos from (Windows' picker, titled `title`): its path, which the
/// companion keeps in the settings (`tv.videos`). None: the player cancelled.
#[tauri::command]
pub async fn pick_tv_videos(window: WebviewWindow, title: String) -> Result<Option<String>, String> {
    let owner = owner(&window);
    blocking(move || Ok(pick::folder(owner, &title).map(|folder| folder.to_string_lossy().into_owned()))).await
}

fn owner(window: &WebviewWindow) -> isize {
    window.hwnd().map(|hwnd| hwnd.0 as isize).unwrap_or(0)
}

/// Runs `work` off the async runtime's threads (pickers, files, decoding images).
async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(|e| e.to_string())?
}
