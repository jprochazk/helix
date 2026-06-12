//! Crude automatic file watching.
//!
//! Every document is polled for external changes periodically and when the
//! terminal regains focus. Unmodified buffers are reloaded from disk;
//! modified buffers are left alone but marked stale, which is rendered as
//! `[+s]` in the statusline and bufferline.

use std::time::Duration;

use helix_view::{doc_mut, document::Mode, view, view_mut, DocumentId, Editor, ViewId};

use crate::job;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub(super) fn setup() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(POLL_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // The first tick completes immediately, skip it.
        interval.tick().await;
        loop {
            interval.tick().await;
            job::dispatch(|editor, _| poll(editor)).await;
        }
    });
}

/// Poll all documents for external changes: reload unmodified buffers from
/// disk, leave modified buffers alone (their recorded disk mtime marks them
/// stale).
pub fn poll(editor: &mut Editor) {
    let scrolloff = editor.config().scrolloff;
    let fallback_view_id = view!(editor).id;
    // The focused document may have a pending (uncommitted) insert-mode
    // transaction; never reload underneath it. It is still polled below, so
    // it gets the stale marker right away.
    let skip_doc_id = (editor.mode() == Mode::Insert).then(|| view!(editor).doc);

    // Mirrors the view-sync dance of `reload_all` in commands/typed.rs.
    let stale_docs: Vec<(DocumentId, Vec<ViewId>)> = editor
        .documents_mut()
        .filter_map(|doc| {
            doc.poll_disk_mtime();
            if skip_doc_id == Some(doc.id()) || !doc.is_stale() || doc.is_modified() {
                return None;
            }

            let mut view_ids: Vec<_> = doc.selections().keys().cloned().collect();
            if view_ids.is_empty() {
                doc.ensure_view_init(fallback_view_id);
                view_ids.push(fallback_view_id);
            }
            Some((doc.id(), view_ids))
        })
        .collect();

    for (doc_id, view_ids) in stale_docs {
        let doc = doc_mut!(editor, &doc_id);
        let view = view_mut!(editor, view_ids[0]);
        view.sync_changes(doc);

        if let Err(err) = doc.reload(view, &editor.diff_providers) {
            // Don't surface errors in the status line: this runs every few
            // seconds and would clobber it.
            log::error!("auto-reload of {:?} failed: {err}", doc.display_name());
            continue;
        }

        if let Some(path) = doc.path().map(ToOwned::to_owned) {
            editor
                .language_servers
                .file_event_handler
                .file_changed(path);
        }

        for view_id in view_ids {
            let view = view_mut!(editor, view_id);
            if view.doc.eq(&doc_id) {
                // Only the first view is synced by the reload itself; sync the
                // rest so their jumplists don't reference stale revisions.
                view.sync_changes(doc);
                view.ensure_cursor_in_view(doc, scrolloff);
            }
        }
    }
}
