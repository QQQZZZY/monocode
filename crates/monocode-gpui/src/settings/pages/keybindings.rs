//! React's command names, defaults, scopes, filter and shortcut recorder.
use crate::{
    settings::{Preferences, SettingsView},
    theme::*,
};
use gpui::{AnyElement, Context, FontWeight, Role, div, prelude::*, px};
#[derive(Clone, Copy)]
pub(crate) struct Binding {
    pub command: &'static str,
    pub label: &'static str,
    pub keys: &'static str,
    pub when: &'static str,
    pub fixed: bool,
}
pub(crate) const BINDINGS: &[Binding] = &[
    Binding {
        command: "App: Settings",
        label: "keybinding-app-settings",
        keys: "⌘,",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: Search",
        label: "keybinding-app-search",
        keys: "⌘K",
        when: "keybinding-scope-always",
        fixed: false,
    },
    #[cfg(target_os = "macos")]
    Binding {
        command: "App: Focus Prompt",
        keys: "⌘L",
        when: "Active chat or session",
        fixed: false,
    },
    Binding {
        command: "App: Go to File",
        label: "keybinding-app-go-to-file",
        keys: "⌘P",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: Command Palette",
        label: "keybinding-app-command-palette",
        keys: "⌘⇧P",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: Find in Files",
        label: "keybinding-app-find-in-files",
        keys: "⌘⇧F",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: Open Project",
        label: "keybinding-app-open-project",
        keys: "⌘O",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: New Window",
        label: "keybinding-app-new-window",
        keys: "⌘⇧N",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: Quick Composer",
        label: "keybinding-app-quick-composer",
        keys: "⌘⇧Space",
        when: "keybinding-scope-anywhere",
        fixed: false,
    },
    Binding {
        command: "App: Toggle Sidebar",
        label: "keybinding-app-toggle-sidebar",
        keys: "⌘B",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: Toggle Session Sidebar",
        label: "keybinding-app-toggle-session-sidebar",
        keys: "⌘⇧B",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: Switch Model",
        label: "keybinding-app-switch-model",
        keys: "⌘.",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "App: Toggle Mono",
        label: "keybinding-app-toggle-mono",
        keys: "⌘I",
        when: "keybinding-scope-project-with-a-mono",
        fixed: false,
    },
    Binding {
        command: "Composer: Toggle Workspace",
        label: "keybinding-composer-toggle-workspace",
        keys: "⌘⇧G",
        when: "keybinding-scope-draft-session-composer",
        fixed: false,
    },
    Binding {
        command: "View: Reload",
        label: "keybinding-view-reload",
        keys: "⌘⇧R",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "View: Zoom In",
        label: "keybinding-view-zoom-in",
        keys: "⌘+",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "View: Zoom Out",
        label: "keybinding-view-zoom-out",
        keys: "⌘-",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "View: Reset Zoom",
        label: "keybinding-view-reset-zoom",
        keys: "⌘0",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: New",
        label: "keybinding-tab-new",
        keys: "⌘T",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Close Others",
        label: "keybinding-tab-close-others",
        keys: "⌘⌥T",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Close All",
        label: "keybinding-tab-close-all",
        keys: "⌘⇧W",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Next",
        label: "keybinding-tab-next",
        keys: "⌘⇧]",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Previous",
        label: "keybinding-tab-previous",
        keys: "⌘⇧[",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Cycle Next",
        label: "keybinding-tab-cycle-next",
        keys: "⌃Tab",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Cycle Previous",
        label: "keybinding-tab-cycle-previous",
        keys: "⌃⇧Tab",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Back",
        label: "keybinding-tab-back",
        keys: "⌘[",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Forward",
        label: "keybinding-tab-forward",
        keys: "⌘]",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Activate 1–8",
        label: "keybinding-tab-activate-1-8",
        keys: "⌘1 … ⌘8",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Tab: Activate Last",
        label: "keybinding-tab-activate-last",
        keys: "⌘9",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Session: Archive",
        label: "keybinding-session-archive",
        keys: "⌘⇧A",
        when: "keybinding-scope-sessionfocus-overlay",
        fixed: false,
    },
    Binding {
        command: "Session: Stop",
        label: "keybinding-session-stop",
        keys: "Esc",
        when: "keybinding-scope-sessionbusy-terminalfocus",
        fixed: true,
    },
    Binding {
        command: "Session: Previous",
        label: "keybinding-session-previous",
        keys: "⌘⇧↑",
        when: "keybinding-scope-overlay-textfocus-emptycomposer",
        fixed: false,
    },
    Binding {
        command: "Session: Next",
        label: "keybinding-session-next",
        keys: "⌘⇧↓",
        when: "keybinding-scope-overlay-textfocus-emptycomposer",
        fixed: false,
    },
    Binding {
        command: "Session: Previous in Current Tab",
        label: "keybinding-session-previous-in-current-tab",
        keys: "⌘↑",
        when: "keybinding-scope-overlay-textfocus-emptycomposer",
        fixed: false,
    },
    Binding {
        command: "Session: Next in Current Tab",
        label: "keybinding-session-next-in-current-tab",
        keys: "⌘↓",
        when: "keybinding-scope-overlay-textfocus-emptycomposer",
        fixed: false,
    },
    Binding {
        command: "Project: Previous",
        label: "keybinding-project-previous",
        keys: "⌘⇧←",
        when: "keybinding-scope-overlay-textfocus-emptycomposer",
        fixed: false,
    },
    Binding {
        command: "Project: Next",
        label: "keybinding-project-next",
        keys: "⌘⇧→",
        when: "keybinding-scope-overlay-textfocus-emptycomposer",
        fixed: false,
    },
    Binding {
        command: "Project: Move Back",
        label: "keybinding-project-move-back",
        keys: "⌘⌥⇧←",
        when: "keybinding-scope-overlay-textfocus-emptycomposer",
        fixed: false,
    },
    Binding {
        command: "Project: Move Forward",
        label: "keybinding-project-move-forward",
        keys: "⌘⌥⇧→",
        when: "keybinding-scope-overlay-textfocus-emptycomposer",
        fixed: false,
    },
    Binding {
        command: "Pane: Close",
        label: "keybinding-pane-close",
        keys: "⌘W",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Pane: Split Right",
        label: "keybinding-pane-split-right",
        keys: "⌘D",
        when: "keybinding-scope-editorfocus",
        fixed: false,
    },
    Binding {
        command: "Pane: Split Down",
        label: "keybinding-pane-split-down",
        keys: "⌘⇧D",
        when: "keybinding-scope-editorfocus",
        fixed: false,
    },
    Binding {
        command: "Pane: Focus Left",
        label: "keybinding-pane-focus-left",
        keys: "⌘⌥←",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Pane: Focus Right",
        label: "keybinding-pane-focus-right",
        keys: "⌘⌥→",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Pane: Focus Up",
        label: "keybinding-pane-focus-up",
        keys: "⌘⌥↑",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Pane: Focus Down",
        label: "keybinding-pane-focus-down",
        keys: "⌘⌥↓",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Terminal: New",
        label: "keybinding-terminal-new",
        keys: "⌘`",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Terminal: New Tab",
        label: "keybinding-terminal-new-tab",
        keys: "⌘⇧`",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Terminal: Toggle Dock",
        label: "keybinding-terminal-toggle-dock",
        keys: "⌘J",
        when: "keybinding-scope-always",
        fixed: false,
    },
    Binding {
        command: "Editor: Find",
        label: "keybinding-editor-find",
        keys: "⌘F",
        when: "keybinding-scope-editorfocus",
        fixed: false,
    },
    Binding {
        command: "Editor: Replace",
        label: "keybinding-editor-replace",
        keys: "⌘⌥F",
        when: "keybinding-scope-editorfocus",
        fixed: false,
    },
];
pub(super) fn quick_composer_label(p: &Preferences) -> String {
    p.keybindings
        .get("App: Quick Composer")
        .and_then(|s| s.clone())
        .map(|s| shortcut_label(&s))
        .unwrap_or_else(|| "⌘⇧Space".into())
}
/// A command's shortcut as shown, honoring the user's rebinding; `None`
/// once they have disabled it.
pub(crate) fn command_shortcut(p: &Preferences, command: &str) -> Option<String> {
    match p.keybindings.get(command) {
        Some(Some(s)) => Some(shortcut_label(s)),
        Some(None) => None,
        None => {
            let keys = BINDINGS.iter().find(|r| r.command == command)?.keys;
            Some(if cfg!(target_os = "macos") {
                keys.into()
            } else {
                keys.replace('⌘', "Ctrl+")
                    .replace('⇧', "Shift+")
                    .replace('⌥', "Alt+")
                    .replace('⌃', "Ctrl+")
            })
        }
    }
}
pub(crate) fn shortcut_label(shortcut: &str) -> String {
    let parts: Vec<_> = shortcut.split('-').collect();
    let mut label = String::new();
    for part in parts {
        label.push_str(&match part {
            "cmd" => "⌘".into(),
            "ctrl" => if cfg!(target_os = "macos") {
                "⌃"
            } else {
                "Ctrl+"
            }
            .into(),
            "alt" => if cfg!(target_os = "macos") {
                "⌥"
            } else {
                "Alt+"
            }
            .into(),
            "shift" => if cfg!(target_os = "macos") {
                "⇧"
            } else {
                "Shift+"
            }
            .into(),
            "space" => "Space".into(),
            "up" => "↑".into(),
            "down" => "↓".into(),
            "left" => "←".into(),
            "right" => "→".into(),
            "tab" => "Tab".into(),
            key => key.to_uppercase(),
        });
    }
    label
}
impl SettingsView {
    pub(super) fn keybindings_page(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let query = self.keybinding_query.trim().to_lowercase();
        let rows: Vec<_> = BINDINGS
            .iter()
            .filter(|r| cfg!(target_os = "macos") || r.command != "App: Quick Composer")
            .filter(|r| {
                format!(
                    "{} {} {}",
                    crate::i18n::text(r.label),
                    self.binding_label(r),
                    crate::i18n::text(r.when)
                )
                .to_lowercase()
                .contains(&query)
            })
            .copied()
            .collect();
        let mut count_args = fluent_bundle::FluentArgs::new();
        count_args.set("count", rows.len());
        count_args.set(
            "countText",
            crate::system_format::integer(rows.len() as i64),
        );
        let count_label = crate::i18n::message("keybindings-count", Some(&count_args));
        let action = div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(content(0.4))
                    .child(count_label),
            )
            .child(
                div()
                    .h(px(28.))
                    .w(px(176.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded_md()
                    .border_1()
                    .border_color(content(0.1))
                    .px_2()
                    .child(icon("search-01", 14.))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .child(self.keybinding_filter.clone()),
                    ),
            )
            .into_any_element();
        let mut children = vec![
            div()
                .flex()
                .items_center()
                .border_b_1()
                .border_color(content(0.1))
                .bg(content(0.05))
                .px_4()
                .py_2()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(content(0.4))
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .child(crate::i18n::text("settings-command")),
                )
                .child(
                    div()
                        .w(px(160.))
                        .flex_shrink_0()
                        .child(crate::i18n::text("settings-keybinding")),
                )
                .child(
                    div()
                        .w(px(112.))
                        .flex_shrink_0()
                        .child(crate::i18n::text("settings-when")),
                )
                .into_any_element(),
        ];
        if rows.is_empty() {
            children.push(super::super::widgets::note(crate::i18n::text(
                "settings-no-matching-bindings",
            )));
        }
        for row in rows {
            let command = row.command;
            let recording = self.recording.as_deref() == Some(command);
            let modified = self.prefs.keybindings.contains_key(command);
            let disabled = self.prefs.keybindings.get(command) == Some(&None);
            let label = if recording {
                if self.recording_preview.is_empty() {
                    crate::i18n::text("settings-record")
                } else {
                    self.recording_preview.clone()
                }
            } else {
                self.binding_label(&row)
            };
            children.push(
                div()
                    .h(px(44.))
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(content(0.05))
                    .px_4()
                    .text_size(px(12.))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .text_color(content(if disabled { 0.45 } else { 1. }))
                            .child(crate::i18n::text(row.label)),
                    )
                    .child(
                        div()
                            .w(px(160.))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap(px(2.))
                            .child(
                                div()
                                    .id(gpui::SharedString::from(format!("shortcut-{command}")))
                                    .debug_selector(move || format!("shortcut-{command}"))
                                    .role(if row.fixed { Role::Label } else { Role::Button })
                                    .aria_label(crate::i18n::format(
                                        "settings-change-shortcut",
                                        &[("command", crate::i18n::text(row.label))],
                                    ))
                                    .h(px(24.))
                                    .w(px(112.))
                                    .flex()
                                    .items_center()
                                    .truncate()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(if recording {
                                        accent_color()
                                    } else {
                                        content(0.15)
                                    })
                                    .px(px(6.))
                                    .font_family(crate::fonts::mono())
                                    .text_size(px(11.))
                                    .text_color(content(if disabled {
                                        0.35
                                    } else if row.fixed {
                                        0.6
                                    } else {
                                        0.8
                                    }))
                                    .child(label)
                                    .when(!row.fixed, |d| {
                                        d.cursor_pointer().hover(|s| s.bg(content(0.1))).on_click(
                                            cx.listener(move |this, _, window, cx| {
                                                this.recording = Some(command.into());
                                                this.recording_preview.clear();
                                                this.recording_error = None;
                                                window.focus(&this.recorder_focus, cx);
                                                cx.notify();
                                            }),
                                        )
                                    }),
                            )
                            .when(modified, |d| {
                                d.child(
                                    div()
                                        .id(gpui::SharedString::from(format!("reset-{command}")))
                                        .debug_selector(move || format!("reset-shortcut-{command}"))
                                        .role(Role::Button)
                                        .aria_label(crate::i18n::format(
                                            "settings-reset-shortcut",
                                            &[("command", crate::i18n::text(row.label))],
                                        ))
                                        .p_1()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(content(0.1)))
                                        .child(icon("rotate-ccw", 14.))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.update(cx, |p| {
                                                p.keybindings.remove(command);
                                            })
                                        })),
                                )
                            }),
                    )
                    .child(
                        div()
                            .w(px(112.))
                            .flex_shrink_0()
                            .truncate()
                            .font_family(crate::fonts::mono())
                            .text_size(px(11.))
                            .text_color(content(0.4))
                            .child(crate::i18n::text(row.when)),
                    )
                    .into_any_element(),
            );
        }
        if let Some(error) = &self.recording_error {
            children.push(
                div()
                    .px_4()
                    .py_2()
                    .text_size(px(11.))
                    .text_color(gpui::rgb(0xf87171))
                    .child(error.clone())
                    .into_any_element(),
            );
        }
        vec![self.group(
            None,
            true,
            crate::i18n::text("settings-shortcuts"),
            Some(crate::i18n::text("settings-click-a-shortcut-to-record-new-keys").into()),
            Some(action),
            children,
        )]
    }
    fn binding_label(&self, row: &Binding) -> String {
        command_shortcut(&self.prefs, row.command)
            .unwrap_or_else(|| crate::i18n::text("settings-disabled"))
    }
    pub(in crate::settings) fn record_key(
        &mut self,
        key: &gpui::Keystroke,
        cx: &mut Context<Self>,
    ) {
        let Some(command) = self.recording.clone() else {
            return;
        };
        cx.stop_propagation();
        if key.key == "escape" || key.key == "tab" {
            self.recording = None;
            cx.notify();
            return;
        }
        if matches!(
            key.key.as_str(),
            "platform" | "control" | "alt" | "shift" | "function"
        ) {
            return;
        }
        let bare = !key.modifiers.platform
            && !key.modifiers.control
            && !key.modifiers.alt
            && !key.modifiers.shift;
        if bare && matches!(key.key.as_str(), "backspace" | "delete") {
            self.recording = None;
            self.update(cx, |p| {
                p.keybindings.insert(command, None);
            });
            return;
        }
        let shortcut = key.unparse();
        self.recording_preview = shortcut_label(&shortcut);
        if bare || (!key.modifiers.platform && !key.modifiers.control && !key.modifiers.alt) {
            self.recording_error = Some(crate::i18n::text(
                "settings-use-command-control-or-option-with-a",
            ));
            cx.notify();
            return;
        }
        if command == "Tab: Activate 1–8"
            && !matches!(
                key.key.as_str(),
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8"
            )
        {
            self.recording_error = Some(crate::i18n::text(
                "settings-tab-activate-1-8-needs-a-number",
            ));
            cx.notify();
            return;
        }
        if let Some(other) = BINDINGS.iter().find(|r| {
            r.command != command
                && (default_shortcuts(r).contains(&shortcut)
                    || self
                        .prefs
                        .keybindings
                        .get(r.command)
                        .and_then(|s| s.as_ref())
                        == Some(&shortcut))
        }) {
            self.recording_error = Some(crate::i18n::format(
                "settings-already-used-by",
                &[("value", (crate::i18n::text(other.label)).to_string())],
            ));
            cx.notify();
            return;
        }
        self.recording = None;
        self.recording_error = None;
        self.update(cx, |p| {
            p.keybindings.insert(command, Some(shortcut));
        });
    }
}

fn default_shortcuts(row: &Binding) -> Vec<String> {
    let shortcut = row
        .keys
        .replace(
            '⌘',
            if cfg!(target_os = "macos") {
                "cmd-"
            } else {
                "ctrl-"
            },
        )
        .replace('⌃', "ctrl-")
        .replace('⌥', "alt-")
        .replace('⇧', "shift-")
        .replace('↑', "up")
        .replace('↓', "down")
        .replace('←', "left")
        .replace('→', "right")
        .replace('+', "=")
        .to_lowercase();
    let chords = if shortcut.contains('…') {
        let prefix = shortcut.split('1').next().unwrap_or("");
        (1..=8).map(|i| format!("{prefix}{i}")).collect()
    } else {
        vec![shortcut]
    };
    chords
        .into_iter()
        .filter_map(|s| gpui::Keystroke::parse(&s).ok().map(|s| s.unparse()))
        .collect()
}
