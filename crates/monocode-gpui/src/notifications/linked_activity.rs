//! LinkedWorkItemUpdateNotice: shell-local card on the active session surface.
use super::github_activity::Card;
use crate::theme::*;
use gpui::{Context, EventEmitter, Render, Role, Window, div, prelude::*, px};
fn activity_summary(card: &Card) -> String {
    let mut parts = Vec::new();
    for (kind, key) in [
        ("commit", "notifications-new-commits"),
        ("review", "notifications-new-reviews"),
        ("comment", "notifications-new-comments"),
    ] {
        let count = card
            .entries
            .iter()
            .filter(|entry| {
                entry.kind == kind || (kind == "comment" && entry.kind == "review_comment")
            })
            .count();
        if count > 0 {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set("count", count);
            args.set("countText", crate::system_format::unsigned(count as u64));
            parts.push(crate::i18n::message(key, Some(&args)));
        }
    }
    if !parts.is_empty() {
        parts.join(" · ")
    } else {
        crate::i18n::text(if card.details_error {
            "notifications-details-unavailable"
        } else {
            "notifications-metadata-or-status-changed"
        })
    }
}

#[derive(Clone)]
pub(crate) enum LinkedActivityEvent {
    Acknowledge(i64),
    Dismiss(i64),
    OpenUrl(String),
    OpenDiscussion(i64),
    AddToChat(i64, String),
    Archive(i64),
    Delete(i64),
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Cleanup {
    Archive,
    Delete,
}
pub(crate) struct LinkedActivity {
    card: Option<Card>,
    cleanup: Option<Cleanup>,
    can_archive: bool,
    can_delete: bool,
}
impl EventEmitter<LinkedActivityEvent> for LinkedActivity {}
impl LinkedActivity {
    pub(crate) fn new() -> Self {
        Self {
            card: None,
            cleanup: None,
            can_archive: false,
            can_delete: false,
        }
    }
    pub(crate) fn set_snapshot(
        &mut self,
        card: Option<Card>,
        can_archive: bool,
        can_delete: bool,
        cx: &mut Context<Self>,
    ) {
        if self.card == card && self.can_archive == can_archive && self.can_delete == can_delete {
            return;
        }
        if self.card.as_ref().map(|c| c.session) != card.as_ref().map(|c| c.session) {
            self.cleanup = None;
        }
        self.card = card;
        self.can_archive = can_archive;
        self.can_delete = can_delete;
        cx.notify();
    }
    /// Archive/delete callbacks acknowledge only after successful completion.
    pub(crate) fn finish_cleanup(&mut self, succeeded: bool, cx: &mut Context<Self>) {
        self.cleanup = None;
        if succeeded && let Some(card) = &self.card {
            cx.emit(LinkedActivityEvent::Acknowledge(card.session));
        }
        cx.notify();
    }
}
impl Render for LinkedActivity {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(card) = self.card.clone() else {
            return div().into_any_element();
        };
        let session = card.session;
        let number = card.item.number;
        let kind = if card.item.kind == "pr" {
            crate::i18n::text("notifications-pull-request")
        } else {
            crate::i18n::text("notifications-issue")
        };
        let latest = card.entries.first();
        let discussion = latest
            .is_some_and(|e| matches!(e.kind.as_str(), "comment" | "review" | "review_comment"));
        let commit = latest.is_some_and(|e| e.kind == "commit");
        let agent_label = if discussion {
            crate::i18n::text("notifications-address-with-agent")
        } else if commit {
            crate::i18n::text("notifications-review-with-agent")
        } else {
            crate::i18n::text("notifications-continue-with-agent")
        };
        let open_label = if commit {
            crate::i18n::text("notifications-open-commit")
        } else if card.item.kind == "pr" {
            crate::i18n::text("notifications-open-pr")
        } else {
            crate::i18n::text("notifications-open-issue")
        };
        let item_url = card.item.url.clone();
        let open_url = latest
            .filter(|e| !e.url.is_empty())
            .map(|e| e.url.clone())
            .unwrap_or_else(|| item_url.clone());
        let prompt = card.prompt();
        let header = div()
            .flex()
            .items_center()
            .gap(px(6.))
            .px(px(12.))
            .py(px(8.))
            .border_b_1()
            .border_color(content(0.1))
            .child(div().size(px(8.)).rounded_full().bg(accent_color()))
            .child(icon(
                if card.item.kind == "pr" {
                    "git-pull-request"
                } else {
                    "circle-dashed"
                },
                14.,
            ))
            .child(
                div()
                    .flex_1()
                    .text_size(px(12.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(crate::i18n::text("notifications-github-activity")),
            )
            .child(
                div()
                    .id("dismiss-linked-activity")
                    .role(Role::Button)
                    .aria_label(crate::i18n::format(
                        "notifications-dismiss-updates-for",
                        &[("kind", kind.to_string()), ("number", number.to_string())],
                    ))
                    .cursor_pointer()
                    .size(px(24.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.emit(LinkedActivityEvent::Acknowledge(session));
                        cx.emit(LinkedActivityEvent::Dismiss(session));
                    }))
                    .child(icon("x", 12.)),
            );
        let mut body = div().relative().child(header).child(
            div()
                .px(px(12.))
                .py(px(10.))
                .child(
                    div()
                        .id("open-linked-work-item")
                        .role(Role::Button)
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(LinkedActivityEvent::Acknowledge(session));
                            cx.emit(LinkedActivityEvent::OpenUrl(item_url.clone()));
                        }))
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(content(0.5))
                                .child(format!("{kind} #{number} · {}", card.item.repo)),
                        )
                        .child(
                            div()
                                .mt(px(2.))
                                .text_size(px(13.))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(card.title.clone()),
                        ),
                )
                .child(
                    div()
                        .mt(px(6.))
                        .text_size(px(11.))
                        .text_color(content(0.55))
                        .child(activity_summary(&card)),
                ),
        );
        for (ix, entry) in card.entries.iter().take(3).enumerate() {
            let url = entry.url.clone();
            let label = match entry.kind.as_str() {
                "commit" => crate::i18n::format(
                    "notifications-commit",
                    &[(
                        "value",
                        (entry.id.chars().take(7).collect::<String>()).to_string(),
                    )],
                ),
                "review" => crate::i18n::text("notifications-review"),
                "review_comment" => crate::i18n::text("notifications-review-comment"),
                _ => crate::i18n::text("notifications-comment"),
            };
            let relative = chrono::DateTime::parse_from_rfc3339(&entry.created_at)
                .ok()
                .map(|at| {
                    let minutes = (chrono::Utc::now().timestamp() - at.timestamp()).max(0) / 60;
                    if minutes < 1 {
                        crate::i18n::text("notifications-just-now")
                    } else if minutes < 60 {
                        crate::i18n::format(
                            "notifications-m-ago",
                            &[("minutes", crate::system_format::unsigned(minutes as u64))],
                        )
                    } else if minutes < 1440 {
                        crate::i18n::format(
                            "notifications-h-ago",
                            &[(
                                "value",
                                crate::system_format::unsigned((minutes / 60) as u64),
                            )],
                        )
                    } else {
                        crate::i18n::format(
                            "notifications-d-ago",
                            &[(
                                "value",
                                crate::system_format::unsigned((minutes / 1440) as u64),
                            )],
                        )
                    }
                })
                .unwrap_or_default();
            body = body.child(
                div()
                    .id(("linked-activity-entry", ix))
                    .role(Role::Button)
                    .px(px(12.))
                    .py(px(8.))
                    .border_t_1()
                    .border_color(content(0.1))
                    .when(!url.is_empty(), |d| {
                        d.cursor_pointer()
                            .hover(|s| s.bg(content(0.05)))
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cx.emit(LinkedActivityEvent::Acknowledge(session));
                                cx.emit(LinkedActivityEvent::OpenUrl(url.clone()));
                            }))
                    })
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .text_size(px(11.))
                            .child(label)
                            .child(div().flex_1().text_color(content(0.45)).child(
                                if entry.author.is_empty() {
                                    String::new()
                                } else {
                                    format!("@{}", entry.author)
                                },
                            ))
                            .child(div().text_color(content(0.35)).child(relative)),
                    )
                    .child(
                        div()
                            .mt(px(2.))
                            .text_size(px(12.))
                            .line_height(px(19.5))
                            .text_color(content(0.65))
                            .child(if entry.text.is_empty() {
                                crate::i18n::text("notifications-no-message").to_owned()
                            } else {
                                entry.text.clone()
                            }),
                    ),
            );
        }
        if let Some(terminal) = card.terminal_label() {
            let busy = self.cleanup.is_some();
            body = body.child(
                div()
                    .px(px(12.))
                    .py(px(10.))
                    .border_t_1()
                    .border_color(content(0.1))
                    .child(div().text_size(px(11.)).text_color(content(0.75)).child(
                        crate::i18n::format(
                            "notifications-clean-up-this-session",
                            &[("terminal", terminal.to_string())],
                        ),
                    ))
                    .child(
                        div()
                            .mt(px(8.))
                            .flex()
                            .gap(px(6.))
                            .child(
                                div()
                                    .id("archive-linked-session")
                                    .role(Role::Button)
                                    .rounded(px(6.))
                                    .px(px(8.))
                                    .py(px(4.))
                                    .bg(content(0.1))
                                    .text_size(px(11.))
                                    .opacity(if busy || !self.can_archive { 0.4 } else { 1. })
                                    .when(!busy && self.can_archive, |d| {
                                        d.cursor_pointer().on_click(cx.listener(
                                            move |this, _, _, cx| {
                                                this.cleanup = Some(Cleanup::Archive);
                                                cx.emit(LinkedActivityEvent::Archive(session));
                                                cx.notify();
                                            },
                                        ))
                                    })
                                    .child(if self.cleanup == Some(Cleanup::Archive) {
                                        crate::i18n::text("notifications-archiving")
                                    } else {
                                        crate::i18n::text("notifications-archive-session")
                                    }),
                            )
                            .child(
                                div()
                                    .id("delete-linked-session")
                                    .role(Role::Button)
                                    .rounded(px(6.))
                                    .px(px(8.))
                                    .py(px(4.))
                                    .text_size(px(11.))
                                    .text_color(gpui::rgb(0xfca5a5))
                                    .opacity(if busy || !self.can_delete { 0.4 } else { 1. })
                                    .when(!busy && self.can_delete, |d| {
                                        d.cursor_pointer().on_click(cx.listener(
                                            move |this, _, _, cx| {
                                                this.cleanup = Some(Cleanup::Delete);
                                                cx.emit(LinkedActivityEvent::Delete(session));
                                                cx.notify();
                                            },
                                        ))
                                    })
                                    .child(if self.cleanup == Some(Cleanup::Delete) {
                                        crate::i18n::text("notifications-deleting")
                                    } else {
                                        crate::i18n::text("notifications-delete")
                                    }),
                            ),
                    ),
            );
        }
        // Solid inverse ink: the translucent pane fill washed out on the light button.
        let mut ink = shell_fill();
        ink.a = 1.;
        body = body.child(
            div()
                .px(px(12.))
                .py(px(10.))
                .border_t_1()
                .border_color(content(0.1))
                .flex()
                .gap(px(6.))
                .text_size(px(11.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(
                    div()
                        .id("linked-activity-agent")
                        .role(Role::Button)
                        .cursor_pointer()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.))
                        .px(px(8.))
                        .py(px(4.))
                        .bg(content(1.))
                        .text_color(ink)
                        .hover(move |s| s.bg(content(0.85)).text_color(ink))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(LinkedActivityEvent::Acknowledge(session));
                            cx.emit(LinkedActivityEvent::AddToChat(session, prompt.clone()));
                        }))
                        .child(agent_label),
                )
                .child(
                    div()
                        .id("linked-activity-open")
                        .role(Role::Button)
                        .cursor_pointer()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.))
                        .px(px(8.))
                        .py(px(4.))
                        .bg(content(0.1))
                        .hover(|s| s.bg(content(0.15)))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(LinkedActivityEvent::Acknowledge(session));
                            if discussion {
                                cx.emit(LinkedActivityEvent::OpenDiscussion(session));
                            } else {
                                cx.emit(LinkedActivityEvent::OpenUrl(open_url.clone()));
                            }
                        }))
                        .child(open_label),
                ),
        );
        super::surface::glass_frame()
            .id("linked-work-item-update-notice")
            .role(Role::Status)
            .aria_label(crate::i18n::format(
                "notifications-new-activity-on",
                &[("kind", kind.to_string()), ("number", number.to_string())],
            ))
            .w(px(320.))
            .child(body)
            .into_any_element()
    }
}
