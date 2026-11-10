//! Port of src/features/settings/model/settings.ts: the pages, their rail
//! groups and the index Settings search jumps through.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Section {
    #[default]
    General,
    Connections,
    Appearance,
    Keybindings,
    Chat,
    Providers,
    Mcp,
    Skills,
    Monos,
    Inbox,
    Archive,
    Worktrees,
}

pub(crate) const GROUPS: [(&str, &[Section]); 3] = [
    (
        "settings-app",
        &[
            Section::General,
            Section::Connections,
            Section::Appearance,
            Section::Keybindings,
        ],
    ),
    (
        "settings-agents",
        &[
            Section::Chat,
            Section::Providers,
            Section::Mcp,
            Section::Skills,
            Section::Monos,
        ],
    ),
    (
        "settings-workspace",
        &[Section::Inbox, Section::Archive, Section::Worktrees],
    ),
];

impl Section {
    pub(crate) const ALL: [Section; 12] = [
        Section::General,
        Section::Connections,
        Section::Appearance,
        Section::Keybindings,
        Section::Chat,
        Section::Providers,
        Section::Mcp,
        Section::Skills,
        Section::Monos,
        Section::Inbox,
        Section::Archive,
        Section::Worktrees,
    ];
    pub(crate) fn id(self) -> &'static str {
        match self {
            Section::General => "general",
            Section::Connections => "connections",
            Section::Appearance => "appearance",
            Section::Keybindings => "keybindings",
            Section::Chat => "chat",
            Section::Providers => "providers",
            Section::Mcp => "mcp",
            Section::Skills => "skills",
            Section::Monos => "monos",
            Section::Inbox => "inbox",
            Section::Archive => "archive",
            Section::Worktrees => "worktrees",
        }
    }
    pub(crate) fn label(self) -> String {
        match self {
            Section::General => crate::i18n::text("settings-general"),
            Section::Connections => crate::i18n::text("settings-connections"),
            Section::Appearance => crate::i18n::text("settings-appearance"),
            Section::Keybindings => crate::i18n::text("settings-keybindings"),
            Section::Chat => crate::i18n::text("settings-chat"),
            Section::Providers => crate::i18n::text("settings-providers"),
            Section::Mcp => crate::i18n::text("settings-mcp"),
            Section::Skills => crate::i18n::text("settings-skills"),
            Section::Monos => crate::i18n::text("settings-monos"),
            Section::Inbox => crate::i18n::text("settings-inbox"),
            Section::Archive => crate::i18n::text("settings-archive"),
            Section::Worktrees => crate::i18n::text("settings-worktrees"),
        }
    }
    pub(crate) fn description(self) -> String {
        match self {
            Section::General => {
                crate::i18n::text("settings-the-build-you-are-running-how-monocode")
            }
            Section::Connections => {
                crate::i18n::text("settings-connect-your-machines-and-run-agents-remotely")
            }
            Section::Appearance => crate::i18n::text(
                "settings-theme-tint-translucency-workspace-layout-and-conversation",
            ),
            Section::Keybindings => {
                crate::i18n::text("settings-every-shortcut-the-workspace-handles-from-the")
            }
            Section::Chat => {
                crate::i18n::text("settings-how-transcripts-read-what-the-composer-does")
            }
            Section::Providers => {
                crate::i18n::text("settings-provider-accounts-agent-clis-monocode-can-drive")
            }
            Section::Mcp => {
                crate::i18n::text("settings-find-mcp-servers-across-providers-and-manage")
            }
            Section::Skills => {
                crate::i18n::text("settings-discover-and-manage-file-skills-from-project")
            }
            Section::Monos => crate::i18n::text("settings-the-resident-agent-beside-your-tabs-and"),
            Section::Inbox => {
                crate::i18n::text("settings-manage-inbox-services-and-notification-preferences-for")
            }
            Section::Archive => {
                crate::i18n::text("settings-projects-and-conversations-you-have-archived")
            }
            Section::Worktrees => {
                crate::i18n::text("settings-manage-additional-worktrees-for-each-project")
            }
        }
    }
    fn keywords(self) -> String {
        match self {
            Section::General => {
                crate::i18n::text("settings-version-update-sounds-notifications-notes-rail")
            }
            Section::Connections => {
                crate::i18n::text("settings-ssh-remote-host-machine-server-environment-always")
            }
            Section::Appearance => {
                crate::i18n::text("settings-theme-dark-light-color-accent-glass-blur")
            }
            Section::Keybindings => crate::i18n::text("settings-shortcut-hotkey-keyboard-binding"),
            Section::Chat => {
                crate::i18n::text("settings-transcript-composer-prompt-message-diff-review-layout")
            }
            Section::Providers => {
                crate::i18n::text("settings-account-sign-in-login-model-harness-claude")
            }
            Section::Mcp => crate::i18n::text(
                "settings-tools-servers-connections-oauth-authenticate-login-claude",
            ),
            Section::Skills => crate::i18n::text("settings-skill-instructions-prompt"),
            Section::Monos => {
                crate::i18n::text("settings-mono-resident-agent-mascot-claim-project-title")
            }
            Section::Inbox => {
                crate::i18n::text("settings-github-gitlab-linear-jira-atlassian-azure-devops")
            }
            Section::Archive => crate::i18n::text("settings-archived-restore-delete-hidden"),
            Section::Worktrees => {
                crate::i18n::text("settings-git-branch-worktree-working-copy-project-create")
            }
        }
    }
    /// SettingsRail.tsx's SECTION_ICONS.
    pub(crate) fn icon(self) -> &'static str {
        match self {
            Section::General => "sliders",
            Section::Connections => "internet",
            Section::Appearance => "palette",
            Section::Keybindings => "keyboard",
            Section::Chat => "message-square",
            Section::Providers => "bot",
            Section::Mcp => "globe",
            Section::Skills => "sparkles",
            Section::Monos => "ghost",
            Section::Inbox => "inbox-hi",
            Section::Archive => "archive",
            Section::Worktrees => "folder-tree",
        }
    }
}

/// SETTINGS_INDEX: (id, section, label, keywords).
pub(crate) const INDEX: &[(&str, Section, &str, &str)] = &[
    (
        "language",
        Section::General,
        "settings-language",
        "settings-search-language",
    ),
    (
        "remote-machines",
        Section::Connections,
        "settings-your-machines",
        "settings-search-remote-machines",
    ),
    (
        "mobile-access",
        Section::Connections,
        "settings-your-phones",
        "settings-search-mobile-access",
    ),
    (
        "mobile-remote",
        Section::Connections,
        "settings-connect-away-from-home",
        "settings-search-mobile-remote",
    ),
    (
        "mcp-servers",
        Section::Mcp,
        "settings-mcp-servers",
        "settings-search-mcp-servers",
    ),
    (
        "monos-enabled",
        Section::Monos,
        "settings-show-monos",
        "settings-search-monos-enabled",
    ),
    (
        "rail-monos-pinned",
        Section::Monos,
        "settings-pin-monos-to-the-icon-rail",
        "settings-search-rail-monos-pinned",
    ),
    (
        "mono-list",
        Section::Monos,
        "settings-your-monos",
        "settings-search-mono-list",
    ),
    (
        "project-worktrees",
        Section::Worktrees,
        "settings-project-worktrees",
        "settings-search-project-worktrees",
    ),
    (
        "update",
        Section::General,
        "settings-version",
        "settings-search-update",
    ),
    (
        "sounds",
        Section::General,
        "settings-sounds",
        "settings-search-sounds",
    ),
    (
        "notifications",
        Section::General,
        "settings-notifications",
        "settings-search-notifications",
    ),
    (
        "notes",
        Section::General,
        "settings-notes",
        "settings-search-notes",
    ),
    (
        "quick-composer",
        Section::General,
        "settings-quick-composer",
        "settings-search-quick-composer",
    ),
    (
        "working-agents",
        Section::General,
        "settings-working-agents",
        "settings-search-working-agents",
    ),
    (
        "action-opens-terminal",
        Section::General,
        "settings-show-terminal-for-actions",
        "settings-search-action-opens-terminal",
    ),
    (
        "terminal-placement",
        Section::General,
        "settings-terminal",
        "settings-search-terminal-placement",
    ),
    (
        "file-tabs",
        Section::General,
        "settings-file-tabs",
        "settings-search-file-tabs",
    ),
    (
        "file-word-wrap",
        Section::General,
        "settings-word-wrap",
        "settings-search-file-word-wrap",
    ),
    (
        "tab-animations",
        Section::General,
        "settings-tab-animations",
        "settings-search-tab-animations",
    ),
    (
        "theme",
        Section::Appearance,
        "settings-theme",
        "settings-search-theme",
    ),
    (
        "wayland-window-decorations",
        Section::Appearance,
        "settings-wayland-window-decorations",
        "settings-search-wayland-window-decorations",
    ),
    (
        "workspace-theme",
        Section::Appearance,
        "settings-workspace-theme",
        "settings-search-workspace-theme",
    ),
    (
        "accent-color",
        Section::Appearance,
        "settings-accent-color",
        "settings-search-accent-color",
    ),
    (
        "diff-colors",
        Section::Appearance,
        "settings-diff-colors",
        "settings-search-diff-colors",
    ),
    (
        "hue",
        Section::Appearance,
        "settings-hue",
        "settings-search-hue",
    ),
    (
        "saturation",
        Section::Appearance,
        "settings-saturation",
        "settings-search-saturation",
    ),
    (
        "dark-lightness",
        Section::Appearance,
        "settings-dark-mode-lightness",
        "settings-search-dark-lightness",
    ),
    (
        "sidebar-opacity",
        Section::Appearance,
        "settings-sidebar-opacity",
        "settings-search-sidebar-opacity",
    ),
    (
        "main-pane-glass",
        Section::Appearance,
        "settings-main-pane-glass",
        "settings-search-main-pane-glass",
    ),
    (
        "inset-layout",
        Section::Appearance,
        "settings-layout",
        "settings-search-inset-layout",
    ),
    (
        "interface-scale",
        Section::Appearance,
        "settings-interface-scale",
        "settings-search-interface-scale",
    ),
    (
        "collapsed-project-rail",
        Section::Appearance,
        "settings-collapsed-project-rail",
        "settings-search-collapsed-project-rail",
    ),
    (
        "show-excluded-files",
        Section::Appearance,
        "settings-show-excluded-files",
        "settings-search-show-excluded-files",
    ),
    (
        "chat-background",
        Section::Appearance,
        "settings-chat-background",
        "settings-search-chat-background",
    ),
    (
        "transcript-layout",
        Section::Chat,
        "settings-transcript-layout",
        "settings-search-transcript-layout",
    ),
    (
        "anchor-prompts",
        Section::Chat,
        "settings-anchor-prompts-to-top",
        "settings-search-anchor-prompts",
    ),
    (
        "paced-reveal",
        Section::Chat,
        "settings-smooth-text-reveal",
        "settings-search-paced-reveal",
    ),
    (
        "collapse-tool-groups",
        Section::Chat,
        "settings-collapse-tool-groups",
        "settings-search-collapse-tool-groups",
    ),
    (
        "fold-finished-work",
        Section::Chat,
        "settings-fold-finished-work",
        "settings-search-fold-finished-work",
    ),
    (
        "follow-up",
        Section::Chat,
        "settings-follow-up-behavior",
        "settings-search-follow-up",
    ),
    (
        "model-controls",
        Section::Chat,
        "settings-model-controls",
        "settings-search-model-controls",
    ),
    (
        "composer-autocorrect",
        Section::Chat,
        "settings-autocorrect",
        "settings-search-composer-autocorrect",
    ),
    (
        "composer-mascot",
        Section::Chat,
        "settings-composer-mascot",
        "settings-search-composer-mascot",
    ),
    (
        "format-on-save",
        Section::Chat,
        "settings-format-on-save",
        "settings-search-format-on-save",
    ),
    (
        "diff-view",
        Section::Chat,
        "settings-diff-view",
        "settings-search-diff-view",
    ),
    (
        "empty-session",
        Section::Chat,
        "settings-empty-session",
        "settings-search-empty-session",
    ),
    (
        "agent-clis",
        Section::Providers,
        "settings-agent-clis",
        "settings-search-agent-clis",
    ),
    (
        "background-model",
        Section::Providers,
        "settings-background-model",
        "settings-search-background-model",
    ),
    (
        "provider-accounts",
        Section::Providers,
        "settings-provider-accounts",
        "settings-search-provider-accounts",
    ),
    (
        "show-remaining-usage",
        Section::Providers,
        "settings-show-remaining-usage",
        "settings-search-show-remaining-usage",
    ),
    (
        "mask-emails",
        Section::Providers,
        "settings-mask-account-emails",
        "settings-search-mask-emails",
    ),
    (
        "claude-hooks",
        Section::Providers,
        "settings-claude-code-hooks",
        "settings-search-claude-hooks",
    ),
    (
        "claude-entrypoint",
        Section::Providers,
        "settings-identify-as-monocode-to-claude",
        "settings-search-claude-entrypoint",
    ),
    (
        "idle-agents",
        Section::Providers,
        "settings-keep-idle-agents-ready",
        "settings-search-idle-agents",
    ),
    (
        "project-notifications",
        Section::Inbox,
        "settings-project-notifications",
        "settings-search-project-notifications",
    ),
    (
        "github",
        Section::Inbox,
        "settings-github",
        "settings-search-github",
    ),
    (
        "gitlab",
        Section::Inbox,
        "settings-gitlab",
        "settings-search-gitlab",
    ),
    (
        "azuredevops",
        Section::Inbox,
        "settings-ado",
        "settings-search-azuredevops",
    ),
    (
        "jira",
        Section::Inbox,
        "settings-jira",
        "settings-search-jira",
    ),
    (
        "linear",
        Section::Inbox,
        "settings-linear",
        "settings-search-linear",
    ),
    (
        "show-archived",
        Section::Archive,
        "settings-show-archived-in-the-sidebar",
        "settings-search-show-archived",
    ),
];

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SearchResult {
    pub section: Section,
    pub setting: Option<&'static str>,
    pub label: String,
}

fn score(needle: &str, label: &str, keywords: &str) -> Option<f32> {
    let lower = label.to_lowercase();
    if lower.starts_with(needle) {
        Some(0.)
    } else if lower.contains(needle) {
        Some(1.)
    } else if keywords.to_lowercase().contains(needle) {
        Some(2.)
    } else {
        None
    }
}

/// searchSettings: individual settings first, then whole sections.
pub(crate) fn search(query: &str, compositor: &str) -> Vec<SearchResult> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(f32, SearchResult)> = Vec::new();
    for &(id, section, label, keywords) in INDEX {
        if id == "wayland-window-decorations" && compositor != "Wayland" {
            continue;
        }
        let label = crate::i18n::text(label);
        let keywords = crate::i18n::text(keywords);
        if let Some(s) = score(&needle, &label, &keywords) {
            scored.push((
                s,
                SearchResult {
                    section,
                    setting: Some(id),
                    label,
                },
            ));
        }
    }
    for section in Section::ALL {
        let keywords = format!("{} {}", section.description(), section.keywords());
        if let Some(s) = score(&needle, &section.label(), &keywords) {
            scored.push((
                s + 0.5,
                SearchResult {
                    section,
                    setting: None,
                    label: section.label(),
                },
            ));
        }
    }
    scored.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| a.1.label.to_lowercase().cmp(&b.1.label.to_lowercase()))
    });
    scored.into_iter().take(8).map(|(_, r)| r).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chinese_startup_search_and_other_threads() {
        const CHILD: &str = "MONOCODE_TEST_LANGUAGE_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "settings::model::tests::chinese_startup_search_and_other_threads",
                ])
                .env(CHILD, "1")
                .status()
                .unwrap();
            assert!(status.success());
            return;
        }
        let workspace = crate::workspace::Workspace::restore(
            Some(r#"{"preferences":{"language":"zh-CN"}}"#),
            &[],
            "/project",
        );
        // This test owns the process-wide startup language on purpose: it
        // checks that a thread which never configures still resolves Chinese
        // and that needs_restart compares against the startup language. Other
        // tests select their own thread's language and stay independent of it.
        crate::i18n::configure(workspace.preferences.language());
        crate::i18n::init();
        let result = search("语言", "")
            .into_iter()
            .find(|row| row.setting == Some("language"))
            .unwrap();
        assert_eq!(result.label, "语言");
        assert_eq!(crate::i18n::text("app-menu-window"), "窗口");
        assert_eq!(
            crate::i18n::text("quick-composer-close-composer"),
            "关闭输入框"
        );
        assert_eq!(
            crate::i18n::format("notice-agent-finished", &[("agent", "Ada".into())]),
            "Ada已完成"
        );
        assert_eq!(
            std::thread::spawn(|| crate::i18n::text("settings-language"))
                .join()
                .unwrap(),
            "语言"
        );
        assert!(!crate::i18n::needs_restart("zh-CN"));
        assert!(crate::i18n::needs_restart("en"));
    }
    #[test]
    fn language_search_uses_the_stable_setting_id() {
        let result = search("language", "")
            .into_iter()
            .find(|row| row.setting == Some("language"))
            .unwrap();
        assert_eq!(result.section, Section::General);
        assert_eq!(result.label, crate::i18n::text("settings-language"));
    }
    #[test]
    fn all_search_rows_share_resolved_english_labels() {
        for &(id, _, label, keywords) in INDEX {
            let label = crate::i18n::text(label);
            assert!(!label.is_empty(), "{id}");
            assert!(!crate::i18n::text(keywords).is_empty(), "{id}");
        }
        let result = search("Sounds", "Wayland")
            .into_iter()
            .find(|row| row.setting == Some("sounds"))
            .unwrap();
        assert_eq!(result.label, "Sounds");
        assert!(
            search("sounds", "Wayland")
                .iter()
                .any(|row| row.section == Section::General)
        );
    }

    #[test]
    fn wayland_decorations_are_searchable_only_on_wayland() {
        for query in ["Wayland", "window decorations", "compositor", "title bar"] {
            let results = search(query, "Wayland");
            let result = results
                .iter()
                .find(|r| r.setting == Some("wayland-window-decorations"))
                .unwrap();
            assert_eq!(result.section, Section::Appearance);
            for compositor in ["X11", "Headless", ""] {
                assert!(
                    search(query, compositor)
                        .iter()
                        .all(|r| r.setting != Some("wayland-window-decorations"))
                );
            }
        }
    }

    #[test]
    fn rows_win_their_own_name_and_shared_appearance_rows_are_searchable() {
        let results = search("theme", "Wayland");
        assert_eq!(results[0].setting, Some("theme"));
        assert_eq!(results[1].setting, Some("workspace-theme"));
        assert!(
            search("hue", "Wayland")
                .iter()
                .any(|r| r.setting == Some("hue"))
        );
        assert!(
            search("workspace theme", "Wayland")
                .iter()
                .any(|r| r.setting == Some("workspace-theme"))
        );
        let pages = search("archive", "Wayland");
        assert!(
            pages
                .iter()
                .any(|r| r.section == Section::Archive && r.setting.is_none())
        );
    }
}
