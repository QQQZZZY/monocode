//! Stable runtime text mapped to the current UI language.
//!
//! Runtime errors also become `status` captions and transcript rows, and those
//! rows are stored. Never translate them where they are produced: match the
//! stable text here, at the display boundary, so switching language re-renders
//! saved rows instead of leaving the old language behind. Provider text,
//! protocol diagnostics and anything else unknown pass through unchanged.

/// A conversation's run status, independent of the UI language.
///
/// `Raw` carries runtime text this table does not know (provider diagnostics,
/// host protocol errors) and UI-thread notices that are already localized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RunStatus {
    Loading,
    Ready,
    Working,
    Connecting,
    Streaming,
    Compacting,
    CreatingWorktree,
    PreparingHandoff,
    PreparingEdit,
    WaitingForApproval,
    WaitingForYourAnswer,
    WorktreeDeleted,
    Stopped,
    WindowClosed,
    StoreUnavailable,
    NotLoaded,
    ProviderNotWaiting,
    NotFoundBranchOrWorktree,
    AccountRemoved,
    AccountSignedOut,
    EditRecoveryRequired,
    Raw(String),
}

/// `(variant, stable runtime text, catalog key)`. Stable text is what workers,
/// storage and external reports carry; English display copy may differ (for
/// example "Session storage is unavailable").
const STATUSES: &[(RunStatus, &str, &str)] = &[
    (
        RunStatus::Loading,
        "Loading conversation…",
        "conversation-loading-conversation",
    ),
    (RunStatus::Ready, "Ready", "conversation-ready"),
    (RunStatus::Working, "Working…", "conversation-working"),
    (
        RunStatus::Connecting,
        "Connecting…",
        "conversation-connecting",
    ),
    (RunStatus::Streaming, "Streaming…", "conversation-streaming"),
    (
        RunStatus::Compacting,
        "Compacting context…",
        "conversation-compacting-context",
    ),
    (
        RunStatus::CreatingWorktree,
        "Creating worktree…",
        "conversation-creating-worktree",
    ),
    (
        RunStatus::PreparingHandoff,
        "Preparing handoff…",
        "conversation-preparing-handoff",
    ),
    (
        RunStatus::PreparingEdit,
        "Preparing edit…",
        "conversation-preparing-edit",
    ),
    (
        RunStatus::WaitingForApproval,
        "Waiting for approval",
        "conversation-waiting-for-approval",
    ),
    (
        RunStatus::WaitingForYourAnswer,
        "Waiting for your answer",
        "conversation-waiting-for-your-answer",
    ),
    (
        RunStatus::WorktreeDeleted,
        "Worktree deleted",
        "conversation-worktree-deleted",
    ),
    (RunStatus::Stopped, "Stopped", "conversation-stopped"),
    (
        RunStatus::WindowClosed,
        "Window closed",
        "conversation-window-closed",
    ),
    (
        RunStatus::StoreUnavailable,
        "Native session store unavailable",
        "conversation-native-session-store-unavailable",
    ),
    (
        RunStatus::NotLoaded,
        "Conversation was not loaded",
        "conversation-was-not-loaded",
    ),
    (
        RunStatus::ProviderNotWaiting,
        "The provider is no longer waiting for this answer",
        "conversation-the-provider-is-no-longer-waiting-for-this-answer",
    ),
    (
        RunStatus::NotFoundBranchOrWorktree,
        "Select a branch or worktree to continue this conversation.",
        "conversation-select-a-branch-or-worktree-to-continue-this",
    ),
    (
        RunStatus::AccountRemoved,
        "This provider account was removed. Choose another account to continue.",
        "conversation-this-provider-account-was-removed-choose-another",
    ),
    (
        RunStatus::AccountSignedOut,
        "This provider account is no longer signed in. Choose another account to continue.",
        "conversation-this-provider-account-is-no-longer-signed-in-choose",
    ),
    (
        RunStatus::EditRecoveryRequired,
        "A previous edit changed provider context before local history could be reconciled. Start a new conversation; the saved history is preserved here.",
        "conversation-edit-recovery-required",
    ),
];

impl RunStatus {
    /// Read stable runtime text. Unknown text stays `Raw` and is resolved by
    /// [`display`] through the provider and remote pipelines.
    pub(crate) fn parse(text: &str) -> Self {
        // A provider reports a cancelled turn with either word.
        if text == "Cancelled" {
            return Self::Stopped;
        }
        STATUSES
            .iter()
            .find_map(|(status, stable, _)| (*stable == text).then_some(status.clone()))
            .unwrap_or_else(|| Self::Raw(text.to_owned()))
    }

    /// Stable English for storage and external reports. Localized `Raw` text
    /// has no stable form and is returned as-is.
    pub(crate) fn stable(&self) -> &str {
        match self {
            Self::Raw(text) => text,
            _ => {
                let (_, stable, _) = STATUSES
                    .iter()
                    .find(|(status, _, _)| status == self)
                    .expect("every non-Raw RunStatus is in STATUSES");
                stable
            }
        }
    }

    pub(crate) fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Monos' "stopped" card also treats a settled stop with a saving failure
    /// appended ("Stopped · Saving failed: …") as stopped, matching the old
    /// prefix check on the report string.
    pub(crate) fn is_stopped(&self) -> bool {
        match self {
            Self::Stopped => true,
            Self::Raw(text) => text.starts_with("Stopped"),
            _ => false,
        }
    }

    /// Account recovery offers a sign-in button for these statuses. Raw text
    /// may be a provider's own sign-in message.
    pub(crate) fn needs_sign_in(&self) -> bool {
        match self {
            Self::AccountRemoved | Self::AccountSignedOut => true,
            Self::Raw(text) => {
                let text = text.to_ascii_lowercase();
                [
                    "not signed in",
                    "not authenticated",
                    "authentication required",
                    "please log in",
                    "please login",
                    "no longer signed in",
                ]
                .iter()
                .any(|phrase| text.contains(phrase))
            }
            _ => false,
        }
    }
}

/// Display text for a run status in the current UI language.
pub(crate) fn display(run: &RunStatus) -> String {
    match run {
        RunStatus::Raw(text) => status(text),
        _ => {
            let (_, _, key) = STATUSES
                .iter()
                .find(|(known, _, _)| known == run)
                .expect("every non-Raw RunStatus is in STATUSES");
            crate::i18n::text(key)
        }
    }
}

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
        "The provider is no longer waiting for this answer" => {
            "conversation-the-provider-is-no-longer-waiting-for-this-answer"
        }
        "Worktree deleted" => "conversation-worktree-deleted",
        "A previous edit changed provider context before local history could be reconciled. Start a new conversation; the saved history is preserved here." => {
            "conversation-edit-recovery-required"
        }
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
        "Too many attachments (20 maximum)" => "conversation-remote-attachment-count",
        _ => return provider(status),
    };
    crate::i18n::text(key)
}

/// Core attachment validation errors; the path stays raw. Resolve on the UI
/// thread and store the result, because the caller's notice is not persisted.
pub(crate) fn attachment_error(text: &str) -> String {
    attachment(text).unwrap_or_else(|| text.to_owned())
}

/// Known attachment-validation sentences. Paths and OS error detail stay raw,
/// so an unreadable file still shows the system's own message.
fn attachment(text: &str) -> Option<String> {
    let key = match text {
        "Attach up to 20 files at a time." => "conversation-attachment-count-limit",
        "Attachments must total 20 MB or less." => "conversation-attachment-size-limit",
        _ => {
            if let Some(path) = text.strip_suffix(" is not a file.") {
                return Some(crate::i18n::format(
                    "conversation-attachment-not-a-file",
                    &[("path", path.to_owned())],
                ));
            }
            // Remote validation omits the period, so the two suffixes do not overlap.
            if let Some(path) = text.strip_suffix(" is not a file") {
                return Some(crate::i18n::format(
                    "conversation-remote-not-a-file",
                    &[("path", path.to_owned())],
                ));
            }
            if let Some(path) =
                text.strip_suffix(" is too large to send to a remote machine (20 MB maximum)")
            {
                return Some(crate::i18n::format(
                    "conversation-remote-too-large",
                    &[("path", path.to_owned())],
                ));
            }
            if let Some(provider) = text.strip_suffix(" is not installed on this machine") {
                return Some(crate::i18n::format(
                    "conversation-remote-provider-missing",
                    &[("provider", provider.to_owned())],
                ));
            }
            return None;
        }
    };
    Some(crate::i18n::text(key))
}

/// Provider availability and account messages produced outside the transcript.
fn provider(status: &str) -> String {
    if let Some(mapped) = attachment(status) {
        return mapped;
    }
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
            "附件过多（最多 20 个）"
        );
        assert_eq!(
            status("Steering is not supported by this remote host"),
            "此远程主机不支持引导。"
        );
        crate::i18n::configure_thread_for_test(Language::English);
    }

    #[test]
    fn every_known_status_round_trips_through_its_stable_text() {
        crate::i18n::configure_thread_for_test(Language::English);
        assert_eq!(STATUSES.len(), 21, "keep the table and the enum in step");
        for (status, stable, key) in STATUSES {
            assert_eq!(
                RunStatus::parse(stable),
                *status,
                "{stable:?} must parse back to its variant"
            );
            assert_eq!(status.stable(), *stable);
            // Display resolves through the catalog, never back through a raw
            // fallback, so a key rename cannot silently drop a variant.
            assert!(!crate::i18n::text(key).is_empty(), "{key}");
        }
        assert_eq!(
            RunStatus::parse("Cancelled"),
            RunStatus::Stopped,
            "a cancelled provider turn settles as stopped"
        );
        assert_eq!(
            RunStatus::parse("Host returned a receipt for another session"),
            RunStatus::Raw("Host returned a receipt for another session".into())
        );
    }

    #[test]
    fn raw_two_word_and_localized_statuses_keep_their_predicates() {
        crate::i18n::configure_thread_for_test(Language::English);
        assert!(RunStatus::Ready.is_ready());
        assert!(!RunStatus::Raw("Couldn't load".into()).is_ready());
        assert!(RunStatus::Stopped.is_stopped());
        assert!(
            RunStatus::parse("Stopped · Saving failed: disk is full").is_stopped(),
            "an appended saving failure still reads as stopped"
        );
        assert!(!RunStatus::Ready.is_stopped());
        assert!(RunStatus::AccountSignedOut.needs_sign_in());
        assert!(RunStatus::AccountRemoved.needs_sign_in());
        assert!(RunStatus::Raw("Please log in to continue".into()).needs_sign_in());
        assert!(!RunStatus::Raw("Couldn't load".into()).needs_sign_in());
        assert!(!RunStatus::Loading.needs_sign_in());
        // Display keeps the display boundary: raw provider detail passes
        // through, while variants resolve to this language's copy.
        assert_eq!(
            display(&RunStatus::Raw("custom provider: raw detail Ω".into())),
            "custom provider: raw detail Ω"
        );
        assert_eq!(display(&RunStatus::Ready), "Ready");
        crate::i18n::configure_thread_for_test(Language::SimplifiedChinese);
        assert_eq!(display(&RunStatus::Ready), "就绪");
        assert_eq!(
            display(&RunStatus::AccountSignedOut),
            "此服务提供商账户已退出登录。请选择其他账户以继续。"
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
