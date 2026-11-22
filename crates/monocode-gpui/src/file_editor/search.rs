//! editorSearch.ts's two-row Find/Replace panel, using the same query engine.
use super::*;
impl FileEditor {
    pub(super) fn search_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let count = if self.search_invalid {
            crate::i18n::text("file-editor-invalid-regex")
        } else if self.search.read(cx).content.is_empty() {
            String::new()
        } else if self.matches.is_empty() {
            crate::i18n::text("file-editor-no-results")
        } else {
            crate::i18n::format(
                "file-editor-of",
                &[
                    (
                        "value",
                        crate::system_format::unsigned((self.match_index + 1) as u64),
                    ),
                    (
                        "value2",
                        crate::system_format::unsigned((self.matches.len().min(999)) as u64),
                    ),
                    (
                        "value3",
                        (if self.matches.len() > 999 { "+" } else { "" }).to_string(),
                    ),
                ],
            )
        };
        let bad = self.search_invalid
            || (!self.search.read(cx).content.is_empty() && self.matches.is_empty());
        let query = div()
            .h(px(26.))
            .min_w_0()
            .flex_1()
            .flex()
            .items_center()
            .px_2()
            .rounded_md()
            .border_1()
            .border_color(if bad {
                rgb(0xf87171).opacity(0.55).into()
            } else {
                content(0.1)
            })
            .bg(content(0.06))
            .child(div().flex_1().min_w_0().child(self.search.clone()))
            .child(
                div()
                    .min_w(px(85.))
                    .text_right()
                    .text_size(px(11.))
                    .text_color(if bad {
                        rgb(0xf87171).into()
                    } else {
                        content(0.45)
                    })
                    .child(count),
            );
        let toggle = |id, label: &str, title: &str, on| {
            div()
                .id(id)
                .role(Role::Button)
                .aria_label(title.to_owned())
                .aria_toggled(if on {
                    gpui::Toggled::True
                } else {
                    gpui::Toggled::False
                })
                .size(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .text_size(px(11.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(content(if on { 1. } else { 0.6 }))
                .when(on, |d| d.bg(accent_color().opacity(0.25)))
                .hover(|s| s.bg(content(0.1)))
                .child(label.to_owned())
        };
        div()
            .flex_shrink_0()
            .flex()
            .items_start()
            .gap_1()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(stroke())
            .font_family(crate::fonts::mono())
            .child(
                icon_action(
                    "toggle-replace",
                    crate::i18n::text("file-editor-toggle-replace"),
                    if self.replace_open {
                        "chevron-down"
                    } else {
                        "chevron-right"
                    },
                    true,
                )
                .mt(px(3.))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.replace_open = !this.replace_open;
                    cx.notify();
                })),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .h(px(26.))
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(query)
                            .child(
                                toggle(
                                    "match-case",
                                    "Aa",
                                    &crate::i18n::text("file-editor-match-case"),
                                    self.match_case,
                                )
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.match_case = !this.match_case;
                                        this.find_matches(cx);
                                    },
                                )),
                            )
                            .child(
                                toggle(
                                    "whole-word",
                                    "ab",
                                    &crate::i18n::text("file-editor-match-whole-word"),
                                    self.whole_word,
                                )
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.whole_word = !this.whole_word;
                                        this.find_matches(cx);
                                    },
                                )),
                            )
                            .child(
                                toggle(
                                    "use-regex",
                                    ".*",
                                    &crate::i18n::text("file-editor-use-regular-expression"),
                                    self.regex,
                                )
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.regex = !this.regex;
                                        this.find_matches(cx);
                                    },
                                )),
                            ),
                    )
                    .when(self.replace_open, |d| {
                        d.child(
                            div()
                                .h(px(26.))
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .h(px(26.))
                                        .flex_1()
                                        .min_w_0()
                                        .px_2()
                                        .flex()
                                        .items_center()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(content(0.1))
                                        .bg(content(0.06))
                                        .child(self.replacement.clone()),
                                )
                                .child(
                                    div()
                                        .id("replace-next")
                                        .role(Role::Button)
                                        .aria_label(crate::i18n::text("file-editor-replace"))
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .text_size(px(11.))
                                        .text_color(content(0.6))
                                        .hover(|s| s.bg(content(0.1)))
                                        .child(crate::i18n::text("file-editor-replace"))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.replace_matches(false, cx)
                                        })),
                                )
                                .child(
                                    div()
                                        .id("replace-all")
                                        .role(Role::Button)
                                        .aria_label(crate::i18n::text("file-editor-replace-all"))
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .text_size(px(11.))
                                        .text_color(content(0.6))
                                        .hover(|s| s.bg(content(0.1)))
                                        .child(crate::i18n::text("file-editor-all"))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.replace_matches(true, cx)
                                        })),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        icon_action(
                            "find-prev",
                            crate::i18n::text("file-editor-previous-match"),
                            "chevron-up",
                            !self.matches.is_empty(),
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.step_match(true, cx))),
                    )
                    .child(
                        icon_action(
                            "find-next",
                            crate::i18n::text("file-editor-next-match"),
                            "chevron-down",
                            !self.matches.is_empty(),
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.step_match(false, cx))),
                    )
                    .child(
                        icon_action(
                            "close-find",
                            crate::i18n::text("file-editor-close"),
                            "x",
                            true,
                        )
                        .size(px(24.))
                        .on_click(cx.listener(|this, _, w, cx| this.dismiss_search(w, cx))),
                    ),
            )
            .into_any_element()
    }
}
