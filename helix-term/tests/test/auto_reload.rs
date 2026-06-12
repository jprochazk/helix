use std::{
    io::Write,
    time::{Duration, SystemTime},
};

use helix_term::application::Application;
use helix_view::{doc, input::parse_macro};
use tempfile::NamedTempFile;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[cfg(windows)]
use crossterm::event::{Event, KeyEvent};
#[cfg(not(windows))]
use termina::event::{Event, KeyEvent};

use super::*;

/// Opens `content` in a new app, with the file's mtime set into the past so
/// that a subsequent external write is detected even on filesystems with
/// coarse mtime granularity.
fn app_with_file_on_disk(content: &str) -> anyhow::Result<(Application, NamedTempFile)> {
    let mut file = NamedTempFile::new()?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    file.as_file()
        .set_modified(SystemTime::now() - Duration::from_secs(60))?;

    let app = helpers::AppBuilder::new()
        .with_file(file.path(), None)
        .build()?;

    Ok((app, file))
}

/// Sends events to the app and runs the event loop until idle.
async fn send_events(app: &mut Application, events: Vec<Event>) -> anyhow::Result<()> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut rx_stream = UnboundedReceiverStream::new(rx);
    for event in events {
        tx.send(Ok(event))?;
    }
    anyhow::ensure!(
        app.event_loop_until_idle(&mut rx_stream).await,
        "app unexpectedly exited"
    );
    Ok(())
}

async fn send_keys(app: &mut Application, keys: &str) -> anyhow::Result<()> {
    let events = parse_macro(keys)?
        .into_iter()
        .map(|key| Event::Key(KeyEvent::from(key)))
        .collect();
    send_events(app, events).await
}

/// Triggers the auto-reload poll by regaining terminal focus.
async fn send_focus_gained(app: &mut Application) -> anyhow::Result<()> {
    #[cfg(windows)]
    let event = Event::FocusGained;
    #[cfg(not(windows))]
    let event = Event::FocusIn;
    send_events(app, vec![event]).await
}

/// Closes the app, asserting a clean exit.
async fn close_app(mut app: Application) -> anyhow::Result<()> {
    test_key_sequence(&mut app, None, None, false).await
}

#[tokio::test(flavor = "multi_thread")]
async fn focus_reloads_unmodified_buffer() -> anyhow::Result<()> {
    let (mut app, file) = app_with_file_on_disk("before\n")?;
    std::fs::write(file.path(), "after\n")?;

    send_focus_gained(&mut app).await?;

    let doc = doc!(app.editor);
    assert_eq!(doc.text().to_string(), "after\n");
    assert!(!doc.is_modified());
    assert!(!doc.is_stale());

    close_app(app).await
}

#[tokio::test(flavor = "multi_thread")]
async fn focus_without_external_change_is_noop() -> anyhow::Result<()> {
    let (mut app, _file) = app_with_file_on_disk("before\n")?;

    send_focus_gained(&mut app).await?;

    let doc = doc!(app.editor);
    assert_eq!(doc.text().to_string(), "before\n");
    assert!(!doc.is_stale());

    close_app(app).await
}

#[tokio::test(flavor = "multi_thread")]
async fn focus_marks_modified_buffer_stale() -> anyhow::Result<()> {
    let (mut app, file) = app_with_file_on_disk("before\n")?;
    send_keys(&mut app, "ihello<esc>").await?;
    std::fs::write(file.path(), "after\n")?;

    send_focus_gained(&mut app).await?;

    // The unsaved changes are kept, but the buffer is marked stale.
    let doc = doc!(app.editor);
    assert_eq!(doc.text().to_string(), "hellobefore\n");
    assert!(doc.is_modified());
    assert!(doc.is_stale());

    close_app(app).await
}

#[tokio::test(flavor = "multi_thread")]
async fn force_write_clears_stale() -> anyhow::Result<()> {
    let (mut app, mut file) = app_with_file_on_disk("before\n")?;
    send_keys(&mut app, "ihello<esc>").await?;
    std::fs::write(file.path(), "after\n")?;
    send_focus_gained(&mut app).await?;
    assert!(doc!(app.editor).is_stale());

    send_keys(&mut app, ":w!<ret>").await?;

    let doc = doc!(app.editor);
    assert!(!doc.is_modified());
    assert!(!doc.is_stale());
    helpers::assert_file_has_content(&mut file, "hellobefore\n")?;

    close_app(app).await
}

#[tokio::test(flavor = "multi_thread")]
async fn force_reload_clears_stale() -> anyhow::Result<()> {
    let (mut app, file) = app_with_file_on_disk("before\n")?;
    send_keys(&mut app, "ihello<esc>").await?;
    std::fs::write(file.path(), "after\n")?;
    send_focus_gained(&mut app).await?;
    assert!(doc!(app.editor).is_stale());

    send_keys(&mut app, ":reload!<ret>").await?;

    let doc = doc!(app.editor);
    assert_eq!(doc.text().to_string(), "after\n");
    assert!(!doc.is_modified());
    assert!(!doc.is_stale());

    close_app(app).await
}

#[tokio::test(flavor = "multi_thread")]
async fn insert_mode_skips_only_the_focused_buffer() -> anyhow::Result<()> {
    let (mut app, other_file) = app_with_file_on_disk("other before\n")?;

    // Open a second file (focusing it) and enter insert mode.
    let mut focused_file = NamedTempFile::new()?;
    focused_file.write_all(b"focused before\n")?;
    focused_file.flush()?;
    focused_file
        .as_file()
        .set_modified(SystemTime::now() - Duration::from_secs(60))?;
    send_keys(
        &mut app,
        &format!(":o {}<ret>i", focused_file.path().display()),
    )
    .await?;

    std::fs::write(other_file.path(), "other after\n")?;
    std::fs::write(focused_file.path(), "focused after\n")?;

    send_focus_gained(&mut app).await?;

    // The focused buffer is not reloaded while in insert mode, but is
    // already marked stale. The other buffer is reloaded.
    let doc = doc!(app.editor);
    assert_eq!(doc.text().to_string(), "focused before\n");
    assert!(doc.is_stale());
    let other_path = helix_stdx::path::canonicalize(other_file.path());
    let other_doc = app
        .editor
        .documents()
        .find(|doc| doc.path() == Some(&other_path))
        .unwrap();
    assert_eq!(other_doc.text().to_string(), "other after\n");
    assert!(!other_doc.is_stale());

    // Leaving insert mode and regaining focus reloads the focused buffer too.
    send_keys(&mut app, "<esc>").await?;
    send_focus_gained(&mut app).await?;
    let doc = doc!(app.editor);
    assert_eq!(doc.text().to_string(), "focused after\n");
    assert!(!doc.is_modified());
    assert!(!doc.is_stale());

    close_app(app).await
}
