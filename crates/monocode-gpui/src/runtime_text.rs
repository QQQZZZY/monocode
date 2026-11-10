//! Stable runtime text mapped to the current UI language.
//!
//! Runtime errors also become `status` captions and transcript rows, and those
//! rows are stored. Never translate them where they are produced: match the
//! stable text here, at the display boundary, so switching language re-renders
//! saved rows instead of leaving the old language behind. Provider text,
//! protocol diagnostics and anything else unknown pass through unchanged.

/// Map a stored runtime status or error to display text.
pub(crate) fn status(status: &str) -> String {
    let key = match status {
        "Working…" => "conversation-working",
        "Ready" => "conversation-ready",
        "Connecting…" => "conversation-connecting",
        "Streaming…" => "conversation-streaming",
        "Creating worktree…" => "conversation-creating-worktree",
        "Preparing handoff…" => "conversation-preparing-handoff",
        "Compacting context…" => "conversation-compacting-context",
        "Waiting for approval" => "conversation-waiting-for-approval",
        "Waiting for your answer" => "conversation-waiting-for-your-answer",
        "Cancelled" | "Stopped" => "conversation-stopped",
        "Window closed" => "conversation-window-closed",
        "Native session store unavailable" => "conversation-native-session-store-unavailable",
        "Conversation was not loaded" => "conversation-was-not-loaded",
        "No completion result was received before the turn ended." => {
            "conversation-no-completion-result"
        }
        "Select a branch or worktree to continue this conversation." => {
            "conversation-select-a-branch-or-worktree-to-continue-this"
        }
        "This provider account was removed. Choose another account to continue." => {
            "conversation-this-provider-account-was-removed-choose-another"
        }
        "Too many attachments (20 maximum)" => "conversation-attachment-limit",
        _ => return remote(status),
    };
    crate::i18n::text(key)
}

/// Remote-session runtime text. Protocol consistency failures stay raw: they
/// describe a host that disagrees with this desktop, which the user cannot act
/// on and which must remain greppable against the host's own diagnostics.
fn remote(status: &str) -> String {
    let key = match status {
        "Choose a model on the remote machine before sending" => {
            "conversation-remote-choose-a-model"
        }
        "Remote connections are unavailable" => "conversation-remote-connections-unavailable",
        "Remote session is unavailable" => "conversation-remote-session-unavailable",
        "Remote session no longer exists" => "conversation-remote-session-gone",
        "Remote session identity is missing" => "conversation-remote-session-identity-missing",
        "Remote session creation has not been confirmed" => {
            "conversation-remote-creation-unconfirmed"
        }
        "Remote attachments have not finished uploading" => {
            "conversation-remote-attachments-uploading"
        }
        "No remote session has been started" => "conversation-remote-no-session-started",
        "Invalid remote attachment" => "conversation-remote-invalid-attachment",
        "Invalid remote worktree" => "conversation-remote-invalid-worktree",
        "Attachment belongs to another machine" => "conversation-remote-attachment-other-machine",
        "Worktree belongs to another machine" => "conversation-remote-worktree-other-machine",
        "Original remote attachment is no longer available" => {
            "conversation-remote-attachment-gone"
        }
        "Attachment changed before upload. Restore the original file before retrying." => {
            "conversation-remote-attachment-changed-before-upload"
        }
        "Attachment changed during upload" => "conversation-remote-attachment-changed",
        "Invalid attachment file name" => "conversation-remote-invalid-attachment-name",
        "Operator and orchestration are not available on remote projects" => {
            "conversation-operator-and-orchestration-are-not-available-on-remote"
        }
        "A remote command needs confirmation. Choose Retry before sending another message." => {
            "conversation-remote-needs-confirmation"
        }
        "Older pending requests need recovery in the original desktop app." => {
            "conversation-remote-legacy-recovery"
        }
        "Steering is not supported by this remote host" => {
            "conversation-remote-steering-unsupported"
        }
        _ => return provider(status),
    };
    crate::i18n::text(key)
}

/// Provider availability and account messages produced outside the transcript.
fn provider(status: &str) -> String {
    if let Some(error) = status.strip_prefix("Saving failed: ") {
        return crate::i18n::format("conversation-saving-failed", &[("error", error.to_owned())]);
    }
    if let Some(rest) = status.strip_prefix("Unknown provider: ") {
        return crate::i18n::format(
            "conversation-unknown-provider",
            &[("provider", rest.to_owned())],
        );
    }
    status.to_owned()
}

/// The code editor's own limit notices. `input::Rejected` carries stable text,
/// so the banner resolves it here instead of storing a translated string.
pub(crate) fn editor_rejection(text: &str) -> String {
    match text {
        "Edit exceeds the 2 MB file limit." => crate::i18n::text("input-edit-exceeds-file-limit"),
        "Edit exceeds the 16 KB line limit." => crate::i18n::text("input-edit-exceeds-line-limit"),
        _ => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn known_runtime_text_localizes_and_unknown_diagnostics_pass_through() {
        crate::i18n::configure_thread_for_test(Language::English);
        assert_eq!(status("Ready"), "Ready");
        assert_eq!(status("Streaming…"), "Streaming…");
        assert_eq!(
            status("custom provider: raw detail Ω"),
            "custom provider: raw detail Ω"
        );
        // A protocol consistency failure is not UX copy.
        assert_eq!(
            status("Host returned a receipt for another session"),
            "Host returned a receipt for another session"
        );

        crate::i18n::configure_thread_for_test(Language::SimplifiedChinese);
        assert_eq!(status("Ready"), "就绪");
        assert_eq!(status("Window closed"), "窗口已关闭");
        assert_eq!(
            status("Select a branch or worktree to continue this conversation."),
            "请选择分支或工作树以继续此对话。"
        );
        assert_eq!(
            status("Too many attachments (20 maximum)"),
            "最多可附加 20 个文件，总计不超过 20 MB。"
        );
        assert_eq!(
            status("Steering is not supported by this remote host"),
            "此远程主机不支持引导。"
        );
        crate::i18n::configure_thread_for_test(Language::English);
    }

    #[test]
    fn saving_failures_keep_the_storage_detail() {
        crate::i18n::configure_thread_for_test(Language::English);
        let mapped = status("Saving failed: disk is full");
        assert!(mapped.contains("disk is full"), "{mapped}");
        assert!(mapped.contains("Saving failed"));
    }

    #[test]
    fn editor_limit_notices_localize_and_other_rejections_pass_through() {
        crate::i18n::configure_thread_for_test(Language::SimplifiedChinese);
        assert_eq!(
            editor_rejection("Edit exceeds the 2 MB file limit."),
            "该编辑过大，无法应用：文件将超过 2 MB。"
        );
        assert_eq!(
            editor_rejection("some provider limit"),
            "some provider limit"
        );
        crate::i18n::configure_thread_for_test(Language::English);
        assert_eq!(
            editor_rejection("Edit exceeds the 16 KB line limit."),
            "This edit is too long to apply: one line would exceed 16 KB."
        );
    }
}
