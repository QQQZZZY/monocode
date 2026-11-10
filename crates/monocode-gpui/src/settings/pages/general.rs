//! GeneralPage and UpdateRow.
use crate::notifications::updater::{self, Action};
use crate::settings::{
    SettingsView,
    widgets::{primary_button, secondary_button},
};
use crate::theme::*;
use gpui::{AnyElement, Context, FontWeight, div, prelude::*, px};

impl SettingsView {
    pub(super) fn general_page(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let p = self.prefs.clone();
        let application = vec![
            self.row(
                Some("language"),
                true,
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(crate::i18n::text("settings-language"))
                    .when(crate::i18n::needs_restart(p.language()), |d| {
                        d.child(
                            div()
                                .debug_selector(|| "settings-language-restart-required".into())
                                .text_size(px(11.))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(content(0.5))
                                .child(crate::i18n::text("settings-restart-required")),
                        )
                    }),
                Some(crate::i18n::text("settings-language-description").into()),
                vec![self.select(
                    "language",
                    crate::i18n::text("settings-language"),
                    p.language(),
                    vec![
                        (
                            "system".into(),
                            crate::i18n::text("settings-system").into(),
                            None,
                        ),
                        ("en".into(), "English".into(), None),
                        ("zh-CN".into(), "简体中文".into(), None),
                    ],
                    Some(160.),
                    cx,
                    |this, value, cx| this.update(cx, |p| p.language = value),
                )],
            ),
        ];
        let alerts = vec![
            self.row(
                Some("sounds"),
                true,
                crate::i18n::text("settings-sounds"),
                Some(
                    crate::i18n::text("settings-play-sounds-for-activity-completed-turns-and")
                        .into(),
                ),
                vec![self.toggle(
                    "sounds",
                    crate::i18n::text("settings-sounds"),
                    p.sounds,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.sounds = on),
                )],
            ),
            self.row(
                Some("notifications"),
                false,
                crate::i18n::text("settings-notifications"),
                Some(crate::i18n::text("settings-alert-when-reminders-are-due-or-agents").into()),
                {
                    let mut controls = self.notification_permission_controls(cx);
                    controls.push(self.toggle(
                        "notifications",
                        crate::i18n::text("settings-notifications"),
                        p.notifications,
                        false,
                        cx,
                        |this, on, cx| {
                            this.update(cx, |p| p.notifications = on);
                            if on {
                                this.probe_notification_permission(true, cx);
                            }
                        },
                    ));
                    controls
                },
            ),
        ];
        let mut workspace = vec![
            self.row(
                Some("file-tabs"),
                true,
                crate::i18n::text("settings-file-tabs"),
                Some(crate::i18n::text("settings-choose-where-files-open").into()),
                vec![self.segmented(
                    "file-tabs",
                    crate::i18n::text("settings-file-tabs"),
                    &p.file_tabs,
                    &[
                        ("pane", &crate::i18n::text("settings-beside-chat")),
                        ("workspace", &crate::i18n::text("settings-top-bar")),
                    ],
                    cx,
                    |this, v, cx| this.update(cx, |p| p.file_tabs = v.into()),
                )],
            ),
            self.row(
                Some("file-word-wrap"),
                false,
                crate::i18n::text("settings-word-wrap"),
                Some(crate::i18n::text("settings-wrap-long-lines-to-fit-the-editor").into()),
                vec![self.toggle(
                    "file-word-wrap",
                    crate::i18n::text("settings-word-wrap"),
                    p.file_word_wrap,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.file_word_wrap = on),
                )],
            ),
            self.row(
                Some("tab-animations"),
                false,
                crate::i18n::text("settings-tab-animations"),
                Some(crate::i18n::text("settings-animate-tabs-as-they-open-and-close").into()),
                vec![self.toggle(
                    "tab-animations",
                    crate::i18n::text("settings-tab-animations"),
                    p.tab_animations,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.tab_animations = on),
                )],
            ),
            self.row(
                Some("notes"),
                false,
                crate::i18n::text("settings-notes"),
                Some(
                    crate::i18n::text("settings-show-notes-in-project-navigation-mention-saved")
                        .into(),
                ),
                vec![self.toggle(
                    "notes",
                    crate::i18n::text("settings-notes"),
                    p.notes,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.notes = on),
                )],
            ),
        ];
        if cfg!(target_os = "macos") {
            workspace.push(
                self.row(
                    Some("quick-composer"),
                    false,
                    crate::i18n::text("settings-quick-composer"),
                    Some(
                        crate::i18n::format(
                            "settings-press-from-any-app-return-starts-a",
                            &[(
                                "value",
                                (super::keybindings::quick_composer_label(&p)).to_string(),
                            )],
                        )
                        .into(),
                    ),
                    {
                        let mut controls = Vec::new();
                        if let Some(error) = &self.ctx.quick_error {
                            controls.push(super::super::widgets::note(error.clone()));
                        }
                        controls.push(self.toggle(
                            "quick-composer",
                            crate::i18n::text("settings-quick-composer"),
                            p.quick_composer,
                            false,
                            cx,
                            |this, on, cx| this.update(cx, |p| p.quick_composer = on),
                        ));
                        controls
                    },
                ),
            );
        }
        workspace.push(self.row(
            Some("search-panel"),
            false,
            crate::i18n::text("settings-search-as-command-panel"),
            Some(
                crate::i18n::text("settings-open-search-as-a-floating-panel-over-your-work").into(),
            ),
            vec![self.toggle(
                "search-panel",
                crate::i18n::text("settings-search-as-command-panel"),
                p.search_panel,
                false,
                cx,
                |this, on, cx| this.update(cx, |p| p.search_panel = on),
            )],
        ));
        workspace.push(
            self.row(
                Some("working-agents"),
                false,
                crate::i18n::text("settings-working-agents"),
                Some(
                    crate::i18n::text("settings-show-active-chats-across-projects-finished-chats")
                        .into(),
                ),
                vec![self.toggle(
                    "working-agents",
                    crate::i18n::text("settings-working-agents"),
                    p.working_agents,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.working_agents = on),
                )],
            ),
        );
        workspace.push(self.row(
            Some("action-opens-terminal"),
            false,
            crate::i18n::text("settings-show-terminal-for-actions"),
            Some(crate::i18n::text("settings-open-the-terminal-when-an-action-runs-off").into()),
            vec![self.toggle(
                "action-opens-terminal",
                crate::i18n::text("settings-show-terminal-for-actions"),
                p.action_opens_terminal,
                false,
                cx,
                |this, on, cx| this.update(cx, |p| p.action_opens_terminal = on),
            )],
        ));
        workspace.push(self.row(
            Some("terminal-placement"),
            false,
            crate::i18n::text("settings-terminal"),
            Some(crate::i18n::text("settings-where-new-terminals-open-docked-below-the").into()),
            vec![self.segmented(
                "terminal-placement",
                crate::i18n::text("settings-terminal"),
                p.terminal_placement(),
                &[
                    ("bottom", &crate::i18n::text("settings-bottom")),
                    ("panel", &crate::i18n::text("settings-side-panel")),
                    ("title", &crate::i18n::text("settings-title-bar")),
                ],
                cx,
                |this, v, cx| this.update(cx, |p| p.set_terminal_placement(v)),
            )],
        ));
        if cfg!(target_os = "windows") {
            workspace.push(self.row(
                Some("close-to-tray"),
                false,
                crate::i18n::text("settings-close-to-tray"),
                Some(crate::i18n::text("settings-closing-a-window-hides-it-to-the").into()),
                vec![self.toggle(
                    "close-to-tray",
                    crate::i18n::text("settings-close-to-tray"),
                    p.close_to_tray,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.close_to_tray = on),
                )],
            ));
        }
        let version = crate::notifications::updater::version();
        let about = vec![
            self.row(
                Some("update"),
                true,
                div()
                    .flex()
                    .items_baseline()
                    .gap_2()
                    .child(crate::i18n::text("settings-version"))
                    .child(
                        div()
                            .font_family(crate::fonts::mono())
                            .text_size(px(12.))
                            .font_weight(FontWeight::NORMAL)
                            .text_color(content(0.45))
                            .child(version),
                    ),
                Some(
                    cx.try_global::<updater::Shared>()
                        .and_then(|shared| updater::status(&shared.0, cx))
                        .unwrap_or_else(|| {
                            crate::i18n::text("settings-view-release-notes-and-check-for-app-and")
                        })
                        .into(),
                ),
                vec![
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            secondary_button(
                                "whats-new",
                                crate::i18n::text("settings-what-s-new"),
                                false,
                                false,
                            )
                            .child(crate::i18n::text("settings-what-s-new"))
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(crate::settings::SettingsEvent::WhatsNew)
                            })),
                        )
                        .child(
                            secondary_button(
                                "check-harness-updates",
                                crate::i18n::text("settings-check-harness-updates"),
                                false,
                                false,
                            )
                            .child(crate::i18n::text("settings-check-harness-updates"))
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(crate::settings::SettingsEvent::CheckHarnessUpdates)
                            })),
                        )
                        .child(self.app_update_button(cx))
                        .into_any_element(),
                ],
            ),
        ];
        vec![
            self.group(
                None,
                true,
                crate::i18n::text("settings-application"),
                None,
                None,
                application,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-alerts"),
                None,
                None,
                alerts,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-workspace"),
                None,
                None,
                workspace,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-about"),
                None,
                None,
                about,
            ),
        ]
    }
}
impl SettingsView {
    /// Check for updates, or once one is found, install it in place.
    fn app_update_button(&self, cx: &mut Context<Self>) -> AnyElement {
        let updater = cx
            .try_global::<updater::Shared>()
            .map(|shared| shared.0.clone());
        match updater.as_ref().map(|u| updater::action(u, cx)) {
            Some(Action::Install(version)) => {
                let label = crate::i18n::format(
                    "settings-update-to",
                    &[("version", (version).to_string())],
                );
                let mut ink = shell_fill();
                ink.a = 1.;
                primary_button("check-updates", label.clone(), false)
                    .child(icon("arrow-down-circle", 14.).text_color(ink))
                    .child(label)
                    .on_click(move |_, _, cx| {
                        if let Some(updater) = &updater {
                            updater::install(updater, cx)
                        }
                    })
                    .into_any_element()
            }
            Some(Action::Busy(label)) => {
                secondary_button("check-updates", label.clone(), false, true)
                    .child(icon("loader", 14.).text_color(content(0.7)))
                    .child(label)
                    .into_any_element()
            }
            Some(Action::Check) | None => secondary_button(
                "check-updates",
                crate::i18n::text("settings-check-for-updates"),
                false,
                false,
            )
            .child(icon("refresh-cw", 14.).text_color(content(0.7)))
            .child(crate::i18n::text("settings-check-for-updates"))
            .on_click(
                cx.listener(|_, _, _, cx| cx.emit(crate::settings::SettingsEvent::CheckAppUpdates)),
            )
            .into_any_element(),
        }
    }
}
