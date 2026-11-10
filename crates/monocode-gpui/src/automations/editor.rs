use super::*;
use chrono::{Local, TimeZone};

pub(super) fn toggle(id: impl Into<gpui::ElementId>, on: bool) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .role(Role::Switch)
        .aria_label(if on {
            crate::i18n::text("automations-pause-automation")
        } else {
            crate::i18n::text("automations-enable-automation")
        })
        .w(px(28.))
        .h(px(16.))
        .flex_shrink_0()
        .rounded_full()
        .px(px(2.))
        .flex()
        .items_center()
        .when(on, |d| d.bg(accent_color()).justify_end())
        .when(!on, |d| d.bg(content(0.2)))
        .child(div().size(px(12.)).rounded_full().bg(rgb(0xffffff)))
}
fn options(values: &[(&str, &str)]) -> Vec<(String, String)> {
    values
        .iter()
        .map(|(v, l)| (v.to_string(), crate::i18n::text(l)))
        .collect()
}
pub(super) fn events(kind: &str) -> Vec<(String, String)> {
    options(match kind {
        "time" => &[
            ("hourly", "automations-hourly"),
            ("daily", "automations-daily"),
            ("weekdays", "automations-weekdays"),
            ("weekly", "automations-weekly"),
        ],
        "github" => &[
            ("draft_opened", "automations-draft-opened"),
            ("pull_request_opened", "automations-pull-request-opened"),
            ("issue_opened", "automations-issue-opened"),
        ],
        "gitlab" => &[
            ("merge_request_opened", "automations-merge-request-opened"),
            ("issue_opened", "automations-issue-opened"),
        ],
        "linear" => &[("issue_created", "automations-issue-created")],
        "jira" => &[("issue_created", "automations-issue-appeared")],
        "azuredevops" => &[
            ("pull_request_appeared", "automations-pull-request-appeared"),
            ("work_item_appeared", "automations-work-item-appeared"),
        ],
        _ => &[],
    })
}
fn event_label(kind: &str, event: &str) -> String {
    let key = match (kind, event) {
        ("jira", "issue_created") => "automations-issue-appeared",
        (_, "draft_opened") => "automations-draft-opened",
        (_, "pull_request_opened") => "automations-pull-request-opened",
        (_, "issue_opened") => "automations-issue-opened",
        (_, "merge_request_opened") => "automations-merge-request-opened",
        (_, "issue_created") => "automations-issue-created",
        (_, "pull_request_appeared") => "automations-pull-request-appeared",
        (_, "work_item_appeared") => "automations-work-item-appeared",
        (_, "pull_request_merged") => "automations-pull-request-merged",
        (_, "pull_request_review_requested") => "automations-review-requested",
        (_, "issue_assigned") => "automations-issue-assigned",
        (_, "check_failed") => "automations-check-failed",
        _ => return event.to_owned(),
    };
    crate::i18n::text(key)
}
pub(super) fn trigger_label(trigger: &AutomationTrigger) -> String {
    if trigger.kind != "time" {
        return event_label(&trigger.kind, &trigger.event);
    }
    let time = crate::system_format::wall_time(&trigger.time);
    match trigger.schedule_kind.as_str() {
        "hourly" => crate::i18n::format(
            "automations-hourly-at",
            &[(
                "minute",
                u64::try_from(trigger.minute)
                    .map(|minute| crate::system_format::padded_unsigned(minute, 2))
                    .unwrap_or_else(|_| format!("{:02}", trigger.minute)),
            )],
        ),
        "daily" => crate::i18n::format("automations-daily-at", &[("time", time)]),
        "weekly" => crate::i18n::format(
            "automations-weekday-at",
            &[
                (
                    "day",
                    if (0..7).contains(&trigger.day_of_week) {
                        crate::system_format::weekday(trigger.day_of_week as usize)
                    } else {
                        crate::i18n::text("automations-weekly")
                    },
                ),
                ("time", time),
            ],
        ),
        _ => crate::i18n::format("automations-weekdays-at", &[("time", time)]),
    }
}

pub(super) fn summary_label(automation: &Automation) -> String {
    let triggers = automation.triggers();
    match triggers.len() {
        0 => crate::i18n::text("automations-manual-only"),
        1 => trigger_label(&triggers[0]),
        count => crate::i18n::format(
            "automations-triggers-more",
            &[
                ("first", trigger_label(&triggers[0])),
                ("count", crate::system_format::unsigned((count - 1) as u64)),
            ],
        ),
    }
}

fn section(label: impl Into<gpui::SharedString>) -> gpui::Div {
    div()
        .px(px(4.))
        .text_size(px(12.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(content(0.5))
        .child(label.into())
}
fn settings_row(
    label: impl Into<gpui::SharedString>,
    hint: impl Into<gpui::SharedString>,
    control: impl IntoElement,
) -> gpui::Div {
    div()
        .min_h(px(54.))
        .px(px(16.))
        .py(px(10.))
        .border_b_1()
        .border_color(content(0.07))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(16.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(content(0.75))
                        .child(label.into()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(content(0.4))
                        .child(hint.into()),
                ),
        )
        .child(control)
}
fn time_options(time: &str) -> Vec<(String, String)> {
    let mut times = (0..24)
        .flat_map(|hour| {
            [0, 30].map(move |minute| {
                let value = format!("{hour:02}:{minute:02}");
                (value.clone(), crate::system_format::wall_time(&value))
            })
        })
        .collect::<Vec<_>>();
    if !times.iter().any(|(value, _)| value == time) {
        times.insert(0, (time.to_owned(), crate::system_format::wall_time(time)));
    }
    times
}

impl Automations {
    fn select_control(
        &self,
        id: impl Into<gpui::ElementId>,
        label: String,
        field: Choice,
        values: Vec<(String, String)>,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        self.select_control_lazy(id, label, field, move || values.clone(), cx)
    }
    fn select_control_lazy(
        &self,
        id: impl Into<gpui::ElementId>,
        label: String,
        field: Choice,
        values: impl Fn() -> Vec<(String, String)> + 'static,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let anchor = AnchorSlot::default();
        let captured = anchor.clone();
        div()
            .id(id)
            .role(Role::Button)
            .aria_label(label.clone())
            .relative()
            .h(px(28.))
            .px(px(8.))
            .rounded(px(6.))
            .bg(content(0.1))
            .flex()
            .items_center()
            .gap(px(4.))
            .text_size(px(12.))
            .text_color(content(1.))
            .hover(|s| s.bg(content(0.14)))
            .child(anchor.probe())
            .when(matches!(field, Choice::Access), |d| {
                let mode = self
                    .editor
                    .as_ref()
                    .map(|e| e.draft.runtime_mode.as_str())
                    .unwrap_or("auto");
                d.child(icon(
                    match mode {
                        "supervised" => "lock",
                        "auto-accept-edits" => "pencil",
                        "full-access" => "shield",
                        _ => "sparkles",
                    },
                    14.,
                ))
            })
            .child(label)
            .child(icon("chevron-down", 12.).text_color(content(0.45)))
            .on_click(cx.listener(move |this, _, _, cx| {
                if matches!(field, Choice::Conversation)
                    && this
                        .editor
                        .as_ref()
                        .is_some_and(|e| e.draft.workspace_mode == "worktree")
                {
                    return;
                }
                this.choice = Some(ChoiceMenu {
                    field: field.clone(),
                    options: values(),
                    anchor: captured.clone(),
                });
                this.trigger_menu = None;
                this.models = None;
                this.project_picker = None;
                cx.notify();
            }))
    }
    pub(super) fn apply_choice(&mut self, field: Choice, value: String, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.editor else {
            return;
        };
        match field {
            Choice::Access => editor.draft.runtime_mode = value,
            Choice::Workspace => {
                editor.draft.workspace_mode = value;
                editor.draft.worktree_cwd.clear();
                if editor.draft.workspace_mode == "worktree" {
                    editor.draft.reuse_session = false;
                }
            }
            Choice::Conversation => editor.draft.reuse_session = value == "reuse",
            Choice::Folder => editor.draft.session_folder_id = value,
            Choice::Grace => editor.draft.missed_run_grace_minutes = value.parse().unwrap_or(720),
            Choice::ModelSetting(key) => {
                set_control_value(&mut editor.draft, key, value);
            }
            Choice::AddTrigger => {
                if let Some((kind, event)) = value.split_once(':') {
                    let triggers = editor.draft.triggers.get_or_insert_with(Vec::new);
                    if triggers.len() < 20 {
                        triggers.push(AutomationTrigger::new(kind, event));
                    }
                }
            }
            Choice::TriggerEvent(id) | Choice::Schedule(id) => {
                if let Some(trigger) = editor
                    .draft
                    .triggers
                    .as_mut()
                    .and_then(|ts| ts.iter_mut().find(|t| t.id == id))
                {
                    trigger.event = value.clone();
                    if trigger.kind == "time" {
                        trigger.schedule_kind = value;
                    }
                }
            }
            Choice::Minute(id) => {
                if let Some(trigger) = editor
                    .draft
                    .triggers
                    .as_mut()
                    .and_then(|ts| ts.iter_mut().find(|t| t.id == id))
                {
                    trigger.minute = value.parse().unwrap_or(0);
                }
            }
            Choice::Time(id) => {
                if let Some(trigger) = editor
                    .draft
                    .triggers
                    .as_mut()
                    .and_then(|ts| ts.iter_mut().find(|t| t.id == id))
                {
                    trigger.time = value.clone();
                }
                if let Some(fields) = editor.fields.iter().find(|f| f.id == id) {
                    fields
                        .time
                        .update(cx, |input, cx| input.set_text(value, cx));
                }
            }
            Choice::Weekday(id) => {
                if let Some(trigger) = editor
                    .draft
                    .triggers
                    .as_mut()
                    .and_then(|ts| ts.iter_mut().find(|t| t.id == id))
                {
                    trigger.day_of_week = value.parse().unwrap_or(1);
                }
            }
        }
        editor.dirty = true;
        self.choice = None;
        self.rebuild_trigger_fields(cx);
        cx.notify();
    }
    pub(super) fn render_model_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(editor) = &self.editor else {
            return div().into_any_element();
        };
        if cx
            .try_global::<crate::settings::Preferences>()
            .is_some_and(|p| p.model_controls == "beside")
            || self.model_controls.is_empty()
        {
            return div().into_any_element();
        }
        div()
            .p(px(8.))
            .border_b_1()
            .border_color(content(0.08))
            .flex()
            .flex_col()
            .gap(px(4.))
            .children(
                self.model_controls
                    .iter()
                    .enumerate()
                    .map(|(index, control)| {
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(12.))
                            .text_size(px(12.))
                            .text_color(content(0.7))
                            .child(control.name.clone())
                            .child(self.select_control(
                                ("automation-model-menu-setting", index),
                                control_label(control, &editor.draft),
                                Choice::ModelSetting(control.id.clone()),
                                control.choices.clone(),
                                cx,
                            ))
                    }),
            )
            .into_any_element()
    }
    fn render_trigger(
        &self,
        trigger: &AutomationTrigger,
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let fields = self
            .editor
            .as_ref()
            .and_then(|e| e.fields.iter().find(|f| f.id == trigger.id));
        let id = trigger.id.clone();
        let mut sentence = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x(px(6.))
            .gap_y(px(4.))
            .text_size(px(13.))
            .text_color(content(0.7));
        if trigger.kind == "time" {
            let prefix = match trigger.schedule_kind.as_str() {
                "hourly" => crate::i18n::text("automations-every-hour-at"),
                "daily" => crate::i18n::text("automations-every-day-at"),
                "weekly" => crate::i18n::text("automations-every-week-on"),
                _ => crate::i18n::text("automations-every-weekday-at"),
            };
            let frequency_anchor = AnchorSlot::default();
            let captured = frequency_anchor.clone();
            let schedule_id = id.clone();
            // The schedule reads as one sentence. Keep frequency editable
            // without adding an extra pill to the sentence.
            sentence = sentence.child(
                div()
                    .id(("trigger-frequency", index))
                    .relative()
                    .role(Role::Button)
                    .aria_label(crate::i18n::format(
                        "automations-schedule",
                        &[("prefix", prefix.to_string())],
                    ))
                    .rounded(px(3.))
                    .hover(|s| s.bg(content(0.05)))
                    .child(frequency_anchor.probe())
                    .child(prefix)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.choice = Some(ChoiceMenu {
                            field: Choice::Schedule(schedule_id.clone()),
                            options: events("time"),
                            anchor: captured.clone(),
                        });
                        cx.notify();
                    })),
            );
            if trigger.schedule_kind == "weekly" {
                sentence = sentence
                    .child(
                        self.select_control(
                            ("trigger-day", index),
                            crate::system_format::weekday(
                                if (0..7).contains(&trigger.day_of_week) {
                                    trigger.day_of_week as usize
                                } else {
                                    1
                                },
                            ),
                            Choice::Weekday(id.clone()),
                            (0..7)
                                .map(|i| (i.to_string(), crate::system_format::weekday(i)))
                                .collect(),
                            cx,
                        ),
                    )
                    .child(crate::i18n::text("automations-at"));
            }
            let mut preview_trigger = trigger.clone();
            if let Some(fields) = fields {
                preview_trigger.time = fields.time.read(cx).content.to_string();
            }
            if trigger.schedule_kind == "hourly" {
                sentence = sentence.child(
                    self.select_control(
                        ("trigger-minute", index),
                        crate::i18n::format(
                            "automations-minute",
                            &[(
                                "minute",
                                u64::try_from(trigger.minute)
                                    .map(|m| crate::system_format::padded_unsigned(m, 2))
                                    .unwrap_or_else(|_| format!("{:02}", trigger.minute)),
                            )],
                        ),
                        Choice::Minute(id.clone()),
                        (0..60)
                            .map(|m| {
                                (
                                    m.to_string(),
                                    crate::i18n::format(
                                        "automations-minute",
                                        &[(
                                            "minute",
                                            crate::system_format::padded_unsigned(m as u64, 2),
                                        )],
                                    ),
                                )
                            })
                            .collect(),
                        cx,
                    ),
                );
            } else {
                let time = preview_trigger.time.clone();
                sentence = sentence.child(self.select_control_lazy(
                    ("trigger-time", index),
                    crate::system_format::wall_time(&time),
                    Choice::Time(id.clone()),
                    move || time_options(&time),
                    cx,
                ));
            }
            sentence = sentence
                .child(div().text_color(content(0.45)).child(gmt_offset()))
                .child(
                    div()
                        .ml(px(4.))
                        .text_color(content(0.35))
                        .child(next_run_preview(
                            preview_trigger.next_at(monocode_core::time::now_ms() as i64),
                        )),
                );
        } else {
            sentence = sentence
                .child(self.select_control(
                    ("trigger-event", index),
                    event_label(&trigger.kind, &trigger.event),
                    Choice::TriggerEvent(id.clone()),
                    events(&trigger.kind),
                    cx,
                ))
                .child(crate::i18n::text("automations-by"));
            if let Some(fields) = fields {
                sentence = sentence.child(
                    div()
                        .w(px(80.))
                        .h(px(28.))
                        .rounded(px(6.))
                        .bg(content(0.1))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .child(fields.actor.clone()),
                );
            }
        }
        let row = div()
            .group("automation-trigger")
            .min_h(px(40.))
            .py(px(4.))
            .flex()
            .items_center()
            .gap(px(10.))
            .child(trigger_mark(&trigger.kind, 14.))
            .child(sentence)
            .child(
                div()
                    .id(SharedString::from(format!("remove-trigger-{id}")))
                    .role(Role::Button)
                    .aria_label(crate::i18n::text("automations-remove-trigger"))
                    .size(px(28.))
                    .flex_shrink_0()
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(content(0.35))
                    .opacity(0.)
                    .group_hover("automation-trigger", |s| s.opacity(1.))
                    .hover(|s| s.bg(content(0.08)).text_color(content(1.)))
                    .child(icon("close", 14.))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(editor) = &mut this.editor {
                            editor
                                .draft
                                .triggers
                                .get_or_insert_with(Vec::new)
                                .retain(|t| t.id != id);
                            editor.dirty = true;
                        }
                        this.rebuild_trigger_fields(cx);
                        cx.notify();
                    })),
            );
        let mut result = div().child(row);
        if trigger.kind != "time"
            && let Some(fields) = fields
        {
            result = result.child(
                div()
                    .pl(px(24.))
                    .pb(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(11.))
                    .text_color(content(0.4))
                    .child(crate::i18n::text("automations-repositories"))
                    .child(div().flex_1().max_w(px(340.)).child(fields.repos.clone())),
            );
        }
        result.into_any_element()
    }
    pub(super) fn render_editor(&self, cx: &mut Context<Self>) -> AnyElement {
        let editor = self.editor.as_ref().expect("editor");
        let draft = &editor.draft;
        let valid = !editor.name.read(cx).content.trim().is_empty()
            && !editor.prompt.read(cx).content.trim().is_empty()
            && !draft.cwd.trim().is_empty();
        let mut actions = div().flex_shrink_0().flex().items_center().gap(px(8.));
        if !editor.saved || editor.dirty {
            actions = actions.child(
                button(
                    "automation-reset",
                    if editor.saved && editor.dirty {
                        crate::i18n::text("automations-reset")
                    } else {
                        crate::i18n::text("automations-prompt-cancel")
                    },
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(id) = this.selected.clone() {
                        this.select(id, cx);
                    } else {
                        this.show_gallery(cx);
                    }
                })),
            );
        }
        if editor.saved {
            actions = actions.child(
                button(
                    "automation-run",
                    if self.running {
                        crate::i18n::text("automations-starting")
                    } else {
                        crate::i18n::text("automations-run-now")
                    },
                )
                .child(icon("queue-play", 13.))
                .on_click(cx.listener(|this, _, _, cx| this.run(cx))),
            );
        }
        let disabled = !valid || self.saving || (editor.saved && !editor.dirty);
        actions = actions.child(
            button(
                "automation-save",
                if self.saving {
                    crate::i18n::text("automations-saving")
                } else if editor.saved {
                    crate::i18n::text("automations-save")
                } else {
                    crate::i18n::text("automations-create")
                },
            )
            .h(px(26.))
            .bg(content(1.))
            .text_color(shell_fill())
            .font_weight(gpui::FontWeight::MEDIUM)
            .when(disabled, |d| d.opacity(0.4))
            .on_click(cx.listener(move |this, _, _, cx| {
                if !disabled {
                    this.save(cx);
                }
            })),
        );
        let header = div()
            .flex_shrink_0()
            .border_b_1()
            .border_color(content(0.07))
            .child(
                div()
                    .w_full()
                    .max_w(px(1024.))
                    .mx_auto()
                    .px(px(32.))
                    .pt(px(20.))
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap(px(24.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child(editor.name.clone()),
                                    ),
                            )
                            .child(actions),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_size(px(12.))
                            .text_color(content(0.5))
                            .child(toggle("automation-enabled", draft.enabled).on_click(
                                cx.listener(|this, _, _, cx| {
                                    if let Some(e) = &mut this.editor {
                                        e.draft.enabled = !e.draft.enabled;
                                        e.dirty = true;
                                    }
                                    cx.notify();
                                }),
                            ))
                            .child(if draft.enabled {
                                crate::i18n::text("automations-active")
                            } else {
                                crate::i18n::text("automations-inactive")
                            })
                            .child(div().ml(px(8.)).h(px(12.)).w(px(1.)).bg(content(0.15)))
                            .child(
                                div()
                                    .id("automation-project")
                                    .relative()
                                    .min_w_0()
                                    .role(Role::Button)
                                    .aria_label(crate::i18n::text(
                                        "automations-choose-automation-project",
                                    ))
                                    .px(px(8.))
                                    .h(px(26.))
                                    .rounded(px(6.))
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .text_size(px(12.))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(content(0.9))
                                    .hover(|s| s.bg(content(0.05)))
                                    .child(self.project_anchor.probe())
                                    .child(self.project_mark(&draft.cwd, 14.))
                                    .child(div().min_w_0().truncate().child(
                                        if draft.cwd.is_empty() {
                                            crate::i18n::text("automations-choose-project")
                                        } else {
                                            self.project_label(&draft.cwd)
                                        },
                                    ))
                                    .child(icon("chevron-down", 12.).text_color(content(0.45)))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.choose_project(window, cx)
                                    })),
                            )
                            .when(editor.saved, |d| {
                                d.child(
                                    icon_button(
                                        "automation-delete-menu",
                                        crate::i18n::text("automations-delete-automation"),
                                        "trash",
                                    )
                                    .on_click(cx.listener(
                                        |this, _, window, cx| this.ask_delete(window, cx),
                                    )),
                                )
                            }),
                    )
                    .child(if editor.saved {
                        div().h(px(36.)).flex().gap(px(16.)).children(
                            [
                                (crate::i18n::text("automations-settings"), false),
                                (crate::i18n::text("automations-run-history"), true),
                            ]
                            .map(|(label, history)| {
                                div()
                                    .id(if history {
                                        "automation-history-tab"
                                    } else {
                                        "automation-settings-tab"
                                    })
                                    .role(Role::Tab)
                                    .aria_label(&label)
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .border_b_2()
                                    .border_color(if self.history == history {
                                        content(0.8)
                                    } else {
                                        content(0.)
                                    })
                                    .text_size(px(12.))
                                    .text_color(content(if self.history == history {
                                        0.9
                                    } else {
                                        0.5
                                    }))
                                    .child(label)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.history = history;
                                        this.choice = None;
                                        cx.notify();
                                    }))
                            }),
                        )
                    } else {
                        div().h(px(20.))
                    }),
            );
        let panel = if self.history && editor.saved {
            self.render_history(cx)
        } else {
            self.render_settings(cx)
        };
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(header)
            .child(panel)
            .into_any_element()
    }
    fn render_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let editor = self.editor.as_ref().expect("editor");
        let draft = &editor.draft;
        let triggers = draft.triggers.clone().unwrap_or_default();
        let triggers_section = div()
            .child(section(crate::i18n::text("automations-triggers")))
            .child(
                div()
                    .mt(px(12.))
                    .rounded(px(6.))
                    .border_1()
                    .border_color(content(0.1))
                    .when(!triggers.is_empty(), |d| {
                        d.child(
                            div().px(px(12.)).py(px(6.)).children(
                                triggers
                                    .iter()
                                    .enumerate()
                                    .map(|(i, t)| self.render_trigger(t, i, cx)),
                            ),
                        )
                        .child(div().h(px(1.)).bg(content(0.08)))
                    })
                    .child(
                        div()
                            .id("automation-add-trigger")
                            .relative()
                            .role(Role::Button)
                            .aria_label(crate::i18n::text("automations-add-trigger"))
                            .h(px(48.))
                            .px(px(12.))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_size(px(13.))
                            .text_color(content(0.55))
                            .hover(|s| s.bg(content(0.04)).text_color(content(1.)))
                            .child(self.trigger_anchor.probe())
                            .child(icon("plus", 14.))
                            .child(crate::i18n::text("automations-add-trigger"))
                            .when(triggers.len() >= 20, |d| d.opacity(0.4))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_trigger_menu(window, cx);
                            })),
                    ),
            );
        let kind = Kind::from_id(&draft.harness);
        let model = monocode_core::store::native_model(&draft.model, &draft.harness);
        let access = match draft.runtime_mode.as_str() {
            "supervised" => crate::i18n::text("automations-supervised"),
            "auto-accept-edits" => crate::i18n::text("automations-auto-accept-edits"),
            "full-access" => crate::i18n::text("automations-full-access"),
            _ => crate::i18n::text("automations-auto"),
        };
        let beside = cx
            .try_global::<crate::settings::Preferences>()
            .is_some_and(|p| p.model_controls == "beside");
        let effort_label = (!beside)
            .then(|| {
                self.model_controls
                    .iter()
                    .find(|control| {
                        matches!(
                            control.id.as_str(),
                            "effort" | "reasoning_effort" | "reasoningEffort" | "reasoning"
                        )
                    })
                    .map(|control| control_label(control, draft))
            })
            .flatten();
        let mut controls = div()
            .px(px(8.))
            .pb(px(8.))
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(4.))
            .child(
                div()
                    .id("automation-model")
                    .relative()
                    .role(Role::Button)
                    .aria_label(crate::i18n::text("automations-choose-automation-model"))
                    .px(px(6.))
                    .h(px(26.))
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .text_size(px(11.))
                    .text_color(content(1.))
                    .bg(content(0.1))
                    .hover(|s| s.bg(content(0.14)))
                    .child(self.model_anchor.probe())
                    .child(provider_icon(kind.id(), 16.))
                    .child(cached_model_name(kind, model.as_deref()))
                    .when_some(effort_label, |d, effort| {
                        d.child(div().text_color(content(0.5)).child(effort))
                    })
                    .child(icon("chevron-down", 12.).text_color(content(0.5)))
                    .on_click(cx.listener(|this, _, window, cx| this.choose_model(window, cx))),
            );
        for (index, control) in self.model_controls.iter().enumerate().filter(|_| beside) {
            controls = controls.child(
                self.select_control(
                    ("automation-model-setting", index),
                    control_label(control, draft),
                    Choice::ModelSetting(control.id.clone()),
                    control.choices.clone(),
                    cx,
                )
                .h(px(26.))
                .px(px(6.))
                .text_size(px(11.)),
            );
        }
        if draft.harness != "fx" {
            controls = controls.child(
                self.select_control(
                    "automation-access",
                    access,
                    Choice::Access,
                    options(&[
                        ("supervised", "automations-supervised"),
                        ("auto-accept-edits", "automations-auto-accept-edits"),
                        ("auto", "automations-auto"),
                        ("full-access", "automations-full-access"),
                    ]),
                    cx,
                )
                .h(px(26.))
                .px(px(6.))
                .text_size(px(11.)),
            );
        }
        let instructions = div()
            .child(section(crate::i18n::text("automations-instructions")))
            .child(
                div()
                    .relative()
                    .mt(px(12.))
                    .rounded(px(6.))
                    .border_1()
                    .border_color(content(0.1))
                    .bg(content(0.03))
                    .child(editor.completion.read(cx).composer_probe())
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(12.))
                            .child(editor.completion.clone()),
                    )
                    .child(controls),
            )
            .child(
                div()
                    .mt(px(8.))
                    .px(px(4.))
                    .text_size(px(11.))
                    .text_color(content(0.35))
                    .child(crate::i18n::text(
                        "automations-skills-file-references-and-built-in",
                    )),
            );
        let mut folder_options = vec![(String::new(), crate::i18n::text("automations-no-folder"))];
        folder_options.extend(self.folders.iter().map(|f| (f.id.clone(), f.name.clone())));
        if !draft.session_folder_id.is_empty()
            && !folder_options
                .iter()
                .any(|(id, _)| id == &draft.session_folder_id)
        {
            folder_options.push((
                draft.session_folder_id.clone(),
                draft.session_folder_id.clone(),
            ));
        }
        let folder_label = folder_options
            .iter()
            .find(|(id, _)| id == &draft.session_folder_id)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| crate::i18n::text("automations-no-folder"));
        let workspace = self.select_control(
            "automation-workspace",
            if draft.workspace_mode == "worktree" {
                crate::i18n::text("automations-fresh-worktree")
            } else if draft.workspace_mode == "existing" {
                crate::i18n::text("automations-existing-worktree")
            } else {
                crate::i18n::text("automations-current")
            },
            Choice::Workspace,
            options(&[
                ("current", "automations-current"),
                ("worktree", "automations-fresh-worktree"),
            ]),
            cx,
        );
        let conversation = self
            .select_control(
                "automation-conversation",
                if draft.reuse_session {
                    crate::i18n::text("automations-continue-last")
                } else {
                    crate::i18n::text("automations-start-fresh")
                },
                Choice::Conversation,
                if draft.workspace_mode == "worktree" {
                    vec![("fresh".into(), crate::i18n::text("automations-start-fresh"))]
                } else {
                    options(&[
                        ("fresh", "automations-start-fresh"),
                        ("reuse", "automations-continue-last"),
                    ])
                },
                cx,
            )
            .when(draft.workspace_mode == "worktree", |d| d.opacity(0.5));
        let session = div()
            .child(section(crate::i18n::text("automations-session")))
            .child(
                div()
                    .mt(px(12.))
                    .rounded(px(6.))
                    .overflow_hidden()
                    .border_1()
                    .border_color(content(0.1))
                    .child(settings_row(
                        crate::i18n::text("automations-working-copy"),
                        crate::i18n::text("automations-this-repo-or-a-fresh-worktree"),
                        workspace,
                    ))
                    .child(settings_row(
                        crate::i18n::text("automations-conversation"),
                        crate::i18n::text("automations-new-chat-or-continue-the-last-run"),
                        conversation,
                    ))
                    .child(
                        settings_row(
                            crate::i18n::text("automations-session-folder"),
                            crate::i18n::text("automations-where-runs-appear-in-the-sidebar"),
                            self.select_control(
                                "automation-folder",
                                folder_label,
                                Choice::Folder,
                                folder_options,
                                cx,
                            ),
                        )
                        .border_b_0(),
                    ),
            );
        let grace_options = options(&[
            ("0", "automations-do-not-catch-up"),
            ("30", "automations-30-minutes"),
            ("120", "automations-2-hours"),
            ("720", "automations-12-hours"),
            ("1440", "automations-24-hours"),
        ]);
        let grace = grace_options
            .iter()
            .find(|(id, _)| *id == draft.missed_run_grace_minutes.to_string())
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| {
                crate::i18n::format(
                    "automations-minutes",
                    &[(
                        "value",
                        crate::system_format::integer(draft.missed_run_grace_minutes),
                    )],
                )
            });
        let advanced = div()
            .rounded(px(6.))
            .border_1()
            .border_color(content(0.1))
            .child(
                div()
                    .id("automation-advanced")
                    .role(Role::Button)
                    .aria_label(crate::i18n::text("automations-advanced"))
                    .min_h(px(56.))
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(3.))
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(content(0.75))
                                    .child(crate::i18n::text("automations-advanced")),
                            )
                            .child(div().text_size(px(11.)).text_color(content(0.4)).child(
                                crate::i18n::text("automations-catch-up-window-for-missed-runs"),
                            )),
                    )
                    .child(icon(
                        if self.advanced {
                            "chevron-up"
                        } else {
                            "chevron-down"
                        },
                        14.,
                    ))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.advanced = !this.advanced;
                        cx.notify();
                    })),
            )
            .when(self.advanced, |d| {
                d.child(settings_row(
                    crate::i18n::text("automations-missed-run-grace"),
                    crate::i18n::text("automations-catch-up-if-a-scheduled-run-was-missed"),
                    self.select_control(
                        "automation-grace",
                        grace,
                        Choice::Grace,
                        grace_options,
                        cx,
                    ),
                ))
            });
        div()
            .id("automation-settings-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                div()
                    .w_full()
                    .max_w(px(1024.))
                    .mx_auto()
                    .px(px(32.))
                    .pt(px(20.))
                    .pb(px(40.))
                    .flex()
                    .flex_col()
                    .gap(px(32.))
                    .child(triggers_section)
                    .child(instructions)
                    .child(session)
                    .child(advanced),
            )
            .into_any_element()
    }
    fn render_history(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut table = div()
            .mt(px(12.))
            .rounded(px(6.))
            .overflow_hidden()
            .border_1()
            .border_color(content(0.1));
        if self.runs.is_empty() {
            table = table.child(
                div()
                    .px(px(16.))
                    .py(px(64.))
                    .text_center()
                    .text_size(px(12.))
                    .text_color(content(0.4))
                    .child(crate::i18n::text(
                        "automations-this-automation-has-not-run-yet",
                    )),
            );
        } else {
            table = table.child(
                div()
                    .h(px(40.))
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .gap(px(16.))
                    .text_size(px(11.))
                    .text_color(content(0.4))
                    .child(
                        div()
                            .flex_1()
                            .child(crate::i18n::text("automations-trigger")),
                    )
                    .child(
                        div()
                            .w(px(152.))
                            .child(crate::i18n::text("automations-triggered")),
                    )
                    .child(
                        div()
                            .w(px(108.))
                            .child(crate::i18n::text("automations-status")),
                    )
                    .child(
                        div()
                            .w(px(56.))
                            .text_right()
                            .child(crate::i18n::text("automations-duration")),
                    ),
            );
            for (index, run) in self.runs.iter().take(100).enumerate() {
                let session = run.session_id.clone();
                let trigger = if run.trigger == "manual" {
                    crate::i18n::text("automations-test-run")
                } else if run.trigger == "event" {
                    event_label(
                        run.event_kind.as_deref().unwrap_or(""),
                        run.event.as_deref().unwrap_or(""),
                    )
                } else {
                    self.editor
                        .as_ref()
                        .and_then(|e| e.draft.triggers.as_ref())
                        .and_then(|ts| ts.iter().find(|t| t.kind == "time"))
                        .map(|t| {
                            crate::i18n::format(
                                "automations-scheduled",
                                &[("name", trigger_label(t))],
                            )
                        })
                        .unwrap_or_else(|| crate::i18n::text("automations-scheduled-2"))
                };
                let (label, color, fill): (String, gpui::Hsla, gpui::Hsla) =
                    match run.status.as_str() {
                        "succeeded" => (
                            crate::i18n::text("automations-succeeded"),
                            rgb(0x34d399).into(),
                            rgb(0x10b981).alpha(0.12).into(),
                        ),
                        "failed" => (
                            crate::i18n::text("automations-failed"),
                            rgb(0xfb7185).into(),
                            rgb(0xf43f5e).alpha(0.12).into(),
                        ),
                        "skipped" => (
                            crate::i18n::text("automations-skipped"),
                            rgb(0xfbbf24).into(),
                            rgb(0xf59e0b).alpha(0.12).into(),
                        ),
                        "cancelled" => (
                            crate::i18n::text("automations-cancelled"),
                            content(0.5),
                            content(0.08),
                        ),
                        "running" => (
                            crate::i18n::text("automations-running"),
                            rgb(0x60a5fa).into(),
                            rgb(0x3b82f6).alpha(0.12).into(),
                        ),
                        _ => (
                            crate::i18n::text("automations-pending"),
                            rgb(0x60a5fa).into(),
                            rgb(0x3b82f6).alpha(0.12).into(),
                        ),
                    };
                table = table.child(
                    div()
                        .id(("automation-run", index))
                        .role(Role::Button)
                        .aria_label(crate::i18n::format(
                            "automations-trigger-label",
                            &[
                                ("trigger", trigger.to_string()),
                                ("label", label.to_string()),
                            ],
                        ))
                        .h(px(44.))
                        .px(px(16.))
                        .border_t_1()
                        .border_color(content(0.08))
                        .flex()
                        .items_center()
                        .gap(px(16.))
                        .text_size(px(13.))
                        .when(session.is_some(), |d| d.hover(|s| s.bg(content(0.05))))
                        .child(div().flex_1().min_w_0().truncate().child(trigger))
                        .child(div().w(px(152.)).text_color(content(0.7)).child(at(
                            if run.scheduled_for == 0 {
                                run.created_at
                            } else {
                                run.scheduled_for
                            },
                        )))
                        .child(
                            div().w(px(108.)).flex().items_center().child(
                                div()
                                    .h(px(20.))
                                    .max_w_full()
                                    .flex_shrink_0()
                                    .px(px(8.))
                                    .flex()
                                    .items_center()
                                    .gap(px(4.))
                                    .rounded_full()
                                    .text_size(px(11.))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(color)
                                    .bg(fill)
                                    .when(run.status == "running", |d| {
                                        d.child(crate::pulse::looping(
                                            SharedString::from(format!(
                                                "automation-run-spinner-{index}"
                                            )),
                                            std::time::Duration::from_secs(1),
                                            crate::pulse::Rate::Smooth,
                                            move |delta| {
                                                icon("loader", 10.)
                                                    .text_color(color)
                                                    .with_transformation(
                                                        gpui::Transformation::rotate(
                                                            gpui::percentage(delta),
                                                        ),
                                                    )
                                            },
                                        ))
                                    })
                                    .child(label),
                            ),
                        )
                        .child(
                            div()
                                .w(px(56.))
                                .text_right()
                                .text_color(content(0.55))
                                .child(duration(run)),
                        )
                        .on_click(cx.listener(move |_, _, _, cx| {
                            if let Some(id) = &session {
                                cx.emit(Event::OpenSession(id.clone()));
                            }
                        })),
                );
                if let Some(error) = &run.error {
                    table = table.child(
                        div()
                            .px(px(16.))
                            .pb(px(8.))
                            .text_size(px(11.))
                            .text_color(content(0.5))
                            .child(error.clone()),
                    );
                }
            }
        }
        div()
            .id("automation-history-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                div()
                    .w_full()
                    .max_w(px(1024.))
                    .mx_auto()
                    .px(px(32.))
                    .pt(px(20.))
                    .pb(px(40.))
                    .child(section(crate::i18n::text("automations-run-history")))
                    .child(table),
            )
            .into_any_element()
    }
}
fn control_label(
    control: &monocode_core::catalog::ModelControl,
    draft: &AutomationUpsert,
) -> String {
    let value = control_value(control, draft);
    let label = control
        .choices
        .iter()
        .find(|(v, _)| v == value)
        .map(|(_, l)| l)
        .unwrap_or(value);
    match label.as_str() {
        "low" => crate::i18n::text("automations-low"),
        "medium" => crate::i18n::text("automations-medium"),
        "high" => crate::i18n::text("automations-high"),
        "xhigh" => crate::i18n::text("automations-extra-high"),
        "minimal" => crate::i18n::text("automations-minimal"),
        "none" => crate::i18n::text("automations-none"),
        _ => label.clone(),
    }
}
fn control_value<'a>(
    control: &'a monocode_core::catalog::ModelControl,
    draft: &'a AutomationUpsert,
) -> &'a String {
    let aliases: &[&str] = match control.id.as_str() {
        "reasoningEffort" | "reasoning_effort" | "effort" | "reasoning" => {
            &["reasoningEffort", "reasoning_effort", "effort", "reasoning"]
        }
        "serviceTier" | "service_tier" => &["serviceTier", "service_tier"],
        _ => &[],
    };
    aliases
        .iter()
        .find_map(|key| draft.model_settings.get(*key))
        .or_else(|| draft.model_settings.get(&control.id))
        .unwrap_or(&control.current)
}
fn set_control_value(draft: &mut AutomationUpsert, key: String, value: String) {
    let (canonical, aliases): (&str, &[&str]) = match key.as_str() {
        "reasoningEffort" | "reasoning_effort" | "effort" | "reasoning" => (
            "reasoningEffort",
            &["reasoningEffort", "reasoning_effort", "effort", "reasoning"],
        ),
        "serviceTier" | "service_tier" => ("serviceTier", &["serviceTier", "service_tier"]),
        _ => (key.as_str(), &[]),
    };
    for alias in aliases {
        draft.model_settings.remove(*alias);
    }
    draft.model_settings.insert(canonical.to_string(), value);
}
fn gmt_offset() -> String {
    let minutes = Local::now().offset().local_minus_utc() / 60;
    let sign = if minutes >= 0 { '+' } else { '-' };
    let absolute = minutes.unsigned_abs();
    if absolute.is_multiple_of(60) {
        format!("GMT{sign}{}", absolute / 60)
    } else {
        format!("GMT{sign}{}:{:02}", absolute / 60, absolute % 60)
    }
}
fn next_run_preview(timestamp: i64) -> String {
    let Some(date) = Local
        .timestamp_millis_opt(timestamp)
        .single()
        .filter(|_| timestamp > 0)
    else {
        return crate::i18n::text("automations-next-run-2");
    };
    let mut zone = gmt_offset();
    #[cfg(unix)]
    {
        let seconds = (timestamp / 1000) as libc::time_t;
        let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
        // Use the system abbreviation (BST, PDT, etc.) for the scheduled date.
        if !unsafe { libc::localtime_r(&seconds, local.as_mut_ptr()) }.is_null() {
            let local = unsafe { local.assume_init() };
            let mut name = [0u8; 64];
            let length = unsafe {
                libc::strftime(name.as_mut_ptr().cast(), name.len(), c"%Z".as_ptr(), &local)
            };
            if length > 0 {
                zone = String::from_utf8_lossy(&name[..length]).into_owned();
            }
        }
    }
    crate::i18n::format(
        "automations-next-run",
        &[
            (
                "value",
                (crate::system_format::date(
                    date.timestamp_millis(),
                    "MMMEdjm",
                    "%a %-d %b, %H:%M",
                ))
                .to_string(),
            ),
            ("zone", (zone).to_string()),
        ],
    )
}
fn at(timestamp: i64) -> String {
    if timestamp <= 0 {
        return "—".into();
    }
    Local
        .timestamp_millis_opt(timestamp)
        .single()
        .map(|d| crate::system_format::date(d.timestamp_millis(), "MMMdjm", "%-d %b, %H:%M"))
        .unwrap_or_else(|| "—".into())
}
fn duration(run: &AutomationRun) -> String {
    let live = matches!(run.status.as_str(), "pending" | "running");
    let start = run
        .started_at
        .or_else(|| (run.status != "pending").then_some(run.created_at));
    let end = run
        .completed_at
        .or_else(|| (live && start.is_some()).then(|| monocode_core::time::now_ms() as i64));
    let (Some(start), Some(end)) = (start, end) else {
        return "—".into();
    };
    if end < start {
        return "—".into();
    }
    let minutes = end.saturating_sub(start) / 60_000;
    if minutes < 1 {
        crate::i18n::text("automations-less-than-minute")
    } else if minutes < 60 {
        crate::i18n::format(
            "automations-m",
            &[("minutes", crate::system_format::unsigned(minutes as u64))],
        )
    } else if minutes % 60 == 0 {
        crate::i18n::format(
            "automations-h",
            &[("value", crate::system_format::integer(minutes / 60))],
        )
    } else {
        crate::i18n::format(
            "duration-hours-minutes",
            &[
                (
                    "whole",
                    crate::system_format::unsigned((minutes / 60) as u64),
                ),
                (
                    "rest",
                    crate::system_format::unsigned((minutes % 60) as u64),
                ),
            ],
        )
    }
}

#[cfg(test)]
mod i18n_tests {
    use super::*;

    #[test]
    fn schedule_captions_preserve_machine_values_and_time_semantics() {
        let mut trigger: AutomationTrigger = serde_json::from_value(
            serde_json::json!({"kind":"time", "scheduleKind":"daily", "time":"09:30"}),
        )
        .unwrap();
        let before = serde_json::to_value(&trigger).unwrap();
        assert_eq!(
            trigger_label(&trigger),
            format!("Daily at {}", crate::system_format::wall_time("09:30"))
        );
        assert_eq!(serde_json::to_value(&trigger).unwrap(), before);
        trigger.schedule_kind = "hourly".into();
        trigger.minute = 5;
        assert_eq!(
            trigger_label(&trigger),
            format!(
                "Hourly at minute {}",
                crate::system_format::padded_unsigned(5, 2)
            )
        );
        trigger.schedule_kind = "weekly".into();
        trigger.day_of_week = 1;
        assert_eq!(
            trigger_label(&trigger),
            format!(
                "{} at {}",
                crate::system_format::weekday(1),
                crate::system_format::wall_time("09:30")
            )
        );
        assert_eq!(event_label("jira", "issue_created"), "Issue appeared");
        assert_eq!(event_label("custom", "custom-event"), "custom-event");
    }
}

#[gpui::test]
fn closed_time_control_does_not_generate_options(cx: &mut gpui::TestAppContext) {
    use std::{cell::Cell, rc::Rc};
    let service = Arc::new(Mutex::new(
        Service::open(std::path::Path::new(":memory:")).unwrap(),
    ));
    let view = cx.new(|cx| Automations::new(service, String::new(), Vec::new(), cx));
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    view.update(cx, |view, cx| {
        let _closed = view.select_control_lazy(
            "time-test",
            crate::system_format::wall_time("09:00"),
            Choice::Time("test".into()),
            move || {
                count.set(count.get() + 1);
                time_options("09:00")
            },
            cx,
        );
    });
    assert_eq!(
        calls.get(),
        0,
        "constructing a closed control must not evaluate its option factory"
    );
    let opened = time_options("09:00");
    assert_eq!(opened.len(), 48);
    assert_eq!(opened[18].0, "09:00");
    let custom = time_options("09:17");
    assert_eq!(custom.len(), 49);
    assert_eq!(custom[0].0, "09:17");
}
