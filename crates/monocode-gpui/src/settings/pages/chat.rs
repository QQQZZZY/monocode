//! ChatPage.
use crate::settings::SettingsView;
use gpui::{AnyElement, Context};

impl SettingsView {
    pub(super) fn chat_page(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let p = self.prefs.clone();
        let transcript = vec![
            self.row(
                Some("transcript-layout"),
                true,
                crate::i18n::text("settings-transcript-layout"),
                Some(crate::i18n::text("settings-show-prompts-at-full-width-or-as").into()),
                vec![self.segmented(
                    "transcript-layout",
                    crate::i18n::text("settings-transcript-layout"),
                    &p.transcript_layout,
                    &[
                        ("full", &crate::i18n::text("settings-full-width")),
                        ("chat", &crate::i18n::text("settings-chat")),
                    ],
                    cx,
                    |this, v, cx| this.update(cx, |p| p.transcript_layout = v.into()),
                )],
            ),
            self.row(
                Some("anchor-prompts"),
                false,
                crate::i18n::text("settings-anchor-prompts-to-top"),
                Some(crate::i18n::text("settings-place-new-prompts-at-the-top-with").into()),
                vec![self.toggle(
                    "anchor-prompts",
                    crate::i18n::text("settings-anchor-prompts-to-top"),
                    p.anchor_prompts,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.anchor_prompts = on),
                )],
            ),
            self.row(
                Some("paced-reveal"),
                false,
                crate::i18n::text("settings-smooth-text-reveal"),
                Some(crate::i18n::text("settings-reveal-incoming-text-gradually").into()),
                vec![self.toggle(
                    "paced-reveal",
                    crate::i18n::text("settings-smooth-text-reveal"),
                    p.paced_reveal,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.paced_reveal = on),
                )],
            ),
            self.row(
                Some("collapse-tool-groups"),
                false,
                crate::i18n::text("settings-collapse-tool-groups"),
                Some(crate::i18n::text("settings-keep-tool-groups-closed-even-while-they").into()),
                vec![self.toggle(
                    "collapse-tool-groups",
                    crate::i18n::text("settings-collapse-tool-groups"),
                    p.collapse_tool_groups,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.collapse_tool_groups = on),
                )],
            ),
            self.row(
                Some("fold-finished-work"),
                false,
                crate::i18n::text("settings-fold-finished-work"),
                Some(crate::i18n::text("settings-fold-a-finished-reply-s-tool").into()),
                vec![self.toggle(
                    "fold-finished-work",
                    crate::i18n::text("settings-fold-finished-work"),
                    p.fold_finished_work,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.fold_finished_work = on),
                )],
            ),
        ];
        let composer = vec![
            self.row(
                Some("follow-up"),
                true,
                crate::i18n::text("settings-follow-up-behavior"),
                Some(crate::i18n::text("settings-queue-follow-ups-until-the-active-turn").into()),
                vec![self.segmented(
                    "follow-up",
                    crate::i18n::text("settings-follow-up-behavior"),
                    &p.follow_up,
                    &[
                        ("queue", &crate::i18n::text("settings-queue")),
                        ("steer", &crate::i18n::text("settings-steer")),
                    ],
                    cx,
                    |this, v, cx| this.update(cx, |p| p.follow_up = v.into()),
                )],
            ),
            self.row(
                Some("model-controls"),
                false,
                crate::i18n::text("settings-model-controls"),
                Some(
                    crate::i18n::text("settings-show-model-options-beside-the-picker-instead")
                        .into(),
                ),
                vec![self.segmented(
                    "model-controls",
                    crate::i18n::text("settings-model-controls"),
                    &p.model_controls,
                    &[
                        ("menu", &crate::i18n::text("settings-menu")),
                        ("beside", &crate::i18n::text("settings-beside")),
                    ],
                    cx,
                    |this, v, cx| this.update(cx, |p| p.model_controls = v.into()),
                )],
            ),
            self.row(
                Some("composer-autocorrect"),
                false,
                crate::i18n::text("settings-autocorrect"),
                Some(crate::i18n::text("settings-spell-check-and-autocorrect-prompts").into()),
                vec![self.toggle(
                    "composer-autocorrect",
                    crate::i18n::text("settings-autocorrect"),
                    p.autocorrect,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.autocorrect = on),
                )],
            ),
        ];
        let editor = vec![
            self.row(
                Some("format-on-save"),
                true,
                crate::i18n::text("settings-format-on-save"),
                Some(
                    crate::i18n::text(
                        "settings-format-supported-files-with-prettier-before-saving",
                    )
                    .into(),
                ),
                vec![self.toggle(
                    "format-on-save",
                    crate::i18n::text("settings-format-on-save"),
                    p.format_on_save,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.format_on_save = on),
                )],
            ),
        ];
        let review = vec![self.row(
            Some("diff-view"),
            true,
            crate::i18n::text("settings-diff-view"),
            Some(crate::i18n::text("settings-review-changes-in-the-editor-or-in").into()),
            vec![self.segmented(
                "diff-view",
                crate::i18n::text("settings-diff-view"),
                &p.diff_view,
                &[
                    ("editor", &crate::i18n::text("settings-editor")),
                    ("unified", &crate::i18n::text("settings-unified")),
                ],
                cx,
                |this, v, cx| this.update(cx, |p| p.diff_view = v.into()),
            )],
        )];
        let extras = vec![
            self.row(
                Some("composer-mascot"),
                true,
                crate::i18n::text("settings-composer-mascot"),
                Some(crate::i18n::text("settings-show-the-project-mascot-while-an-agent").into()),
                vec![self.toggle(
                    "composer-mascot",
                    crate::i18n::text("settings-composer-mascot"),
                    p.composer_mascot,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.composer_mascot = on),
                )],
            ),
            self.row(
                Some("empty-session"),
                false,
                crate::i18n::text("settings-empty-session"),
                Some(crate::i18n::text("settings-stir-dithered-ink-play-a-game-or").into()),
                vec![self.segmented(
                    "empty-session",
                    crate::i18n::text("settings-empty-session"),
                    &p.empty_session,
                    &[
                        ("ink", &crate::i18n::text("settings-ink")),
                        ("games", &crate::i18n::text("settings-games")),
                        ("none", &crate::i18n::text("settings-off")),
                    ],
                    cx,
                    |this, v, cx| this.update(cx, |p| p.empty_session = v.into()),
                )],
            ),
        ];
        vec![
            self.group(
                None,
                true,
                crate::i18n::text("settings-transcript"),
                None,
                None,
                transcript,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-composer"),
                None,
                None,
                composer,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-editor"),
                None,
                None,
                editor,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-code-review"),
                None,
                None,
                review,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-extras"),
                None,
                None,
                extras,
            ),
        ]
    }
}
