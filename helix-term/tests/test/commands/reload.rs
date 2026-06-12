use std::io::Write;

use helix_core::diagnostic::Severity;
use helix_view::{doc, Editor};

use super::*;

fn new_file_on_disk(content: &str) -> anyhow::Result<tempfile::NamedTempFile> {
    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    Ok(file)
}

fn assert_status_error(editor: &Editor) {
    match editor.get_status() {
        Some((_, &Severity::Error)) => {}
        status => panic!("expected error status, got {:?}", status),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reload_unmodified_buffer() -> anyhow::Result<()> {
    let file = new_file_on_disk("before\n")?;
    let mut app = helpers::AppBuilder::new()
        .with_file(file.path(), None)
        .build()?;

    std::fs::write(file.path(), "after\n")?;

    test_key_sequence(
        &mut app,
        Some(":reload<ret>"),
        Some(&|app| {
            helpers::assert_status_not_error(&app.editor);
            assert_eq!(doc!(app.editor).text().to_string(), "after\n");
        }),
        false,
    )
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reload_fails_on_modified_buffer() -> anyhow::Result<()> {
    let file = new_file_on_disk("before\n")?;
    let mut app = helpers::AppBuilder::new()
        .with_file(file.path(), None)
        .build()?;

    std::fs::write(file.path(), "after\n")?;

    test_key_sequence(
        &mut app,
        Some("ihello<esc>:reload<ret>"),
        Some(&|app| {
            assert_status_error(&app.editor);
            // The unsaved changes are kept.
            let doc = doc!(app.editor);
            assert!(doc.is_modified());
            assert_eq!(doc.text().to_string(), "hellobefore\n");
        }),
        false,
    )
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_force_reload_modified_buffer() -> anyhow::Result<()> {
    let file = new_file_on_disk("before\n")?;
    let mut app = helpers::AppBuilder::new()
        .with_file(file.path(), None)
        .build()?;

    std::fs::write(file.path(), "after\n")?;

    test_key_sequence(
        &mut app,
        Some("ihello<esc>:reload!<ret>"),
        Some(&|app| {
            helpers::assert_status_not_error(&app.editor);
            assert_eq!(doc!(app.editor).text().to_string(), "after\n");
        }),
        false,
    )
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reload_all_skips_modified_buffer() -> anyhow::Result<()> {
    let file = new_file_on_disk("before\n")?;
    let mut app = helpers::AppBuilder::new()
        .with_file(file.path(), None)
        .build()?;

    std::fs::write(file.path(), "after\n")?;

    test_key_sequence(
        &mut app,
        Some("ihello<esc>:reload-all<ret>"),
        Some(&|app| {
            assert_status_error(&app.editor);
            // The unsaved changes are kept.
            let doc = doc!(app.editor);
            assert!(doc.is_modified());
            assert_eq!(doc.text().to_string(), "hellobefore\n");
        }),
        false,
    )
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_force_reload_all_modified_buffer() -> anyhow::Result<()> {
    let file = new_file_on_disk("before\n")?;
    let mut app = helpers::AppBuilder::new()
        .with_file(file.path(), None)
        .build()?;

    std::fs::write(file.path(), "after\n")?;

    test_key_sequence(
        &mut app,
        Some("ihello<esc>:reload-all!<ret>"),
        Some(&|app| {
            helpers::assert_status_not_error(&app.editor);
            assert_eq!(doc!(app.editor).text().to_string(), "after\n");
        }),
        false,
    )
    .await?;

    Ok(())
}
