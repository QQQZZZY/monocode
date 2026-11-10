use crate::{
    settings::{
        SettingsEvent, SettingsView,
        widgets::{note, secondary_button},
    },
    theme::*,
};
use gpui::{AnyElement, Context, Role, SharedString, div, prelude::*, px};
impl SettingsView {
    pub(super) fn archive_page(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut rows = vec![
            self.row(
                Some("show-archived"),
                true,
                crate::i18n::text("settings-show-archived-in-the-sidebar"),
                Some(
                    crate::i18n::text(
                        "settings-keep-archived-conversations-listed-alongside-the-active",
                    )
                    .into(),
                ),
                vec![self.toggle(
                    "show-archived",
                    crate::i18n::text("settings-show-archived-in-the-sidebar"),
                    self.prefs.show_archived,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.show_archived = on),
                )],
            ),
        ];
        if self.ctx.cwd.is_empty() {
            rows.push(note(crate::i18n::text(
                "settings-open-a-project-to-see-its-archived",
            )));
        } else if self.ctx.archived.is_empty() {
            rows.push(note(crate::i18n::text(
                "settings-no-archived-conversations-in-this-project",
            )));
        }
        for session in &self.ctx.archived {
            let id = session.id;
            let date = archive_date(session.updated_at);
            rows.push(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .border_t_1()
                    .border_color(content(0.05))
                    .px_4()
                    .py(px(10.))
                    .child(provider_icon(&session.provider, 14.))
                    .child(
                        div()
                            .id(SharedString::from(format!("archived-session-{id}")))
                            .role(Role::Button)
                            .aria_label(session.title.clone())
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .text_size(px(13.))
                            .cursor_pointer()
                            .child(session.title.clone())
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cx.emit(SettingsEvent::OpenSession(id))
                            })),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(content(0.35))
                            .child(date),
                    )
                    .child(
                        secondary_button(
                            SharedString::from(format!("unarchive-{id}")),
                            crate::i18n::text("settings-unarchive"),
                            false,
                            false,
                        )
                        .child(crate::i18n::text("settings-unarchive"))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(SettingsEvent::Archive(id, false))
                        })),
                    )
                    .child(
                        secondary_button(
                            SharedString::from(format!("delete-{id}")),
                            crate::i18n::text("settings-delete"),
                            true,
                            false,
                        )
                        .child(crate::i18n::text("settings-delete"))
                        .on_click(
                            cx.listener(move |_, _, _, cx| cx.emit(SettingsEvent::Delete(id))),
                        ),
                    )
                    .into_any_element(),
            );
        }
        let title = if self.ctx.cwd.is_empty() {
            crate::i18n::text("settings-archived-conversations")
        } else {
            crate::i18n::format(
                "settings-archived-in",
                &[("value", (self.ctx.project_label).to_string())],
            )
        };
        let mut projects: Vec<AnyElement> = self
            .ctx
            .archived_projects
            .iter()
            .enumerate()
            .map(|(ix, (path, label))| {
                let (unarchive, delete) = (path.clone(), path.clone());
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .when(ix > 0, |d| d.border_t_1().border_color(content(0.05)))
                    .px_4()
                    .py(px(10.))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(div().truncate().text_size(px(13.)).child(label.clone()))
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(11.))
                                    .text_color(content(0.35))
                                    .child(path.clone()),
                            ),
                    )
                    .child(
                        secondary_button(
                            SharedString::from(format!("unarchive-project-{ix}")),
                            crate::i18n::text("settings-unarchive"),
                            false,
                            false,
                        )
                        .child(crate::i18n::text("settings-unarchive"))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(SettingsEvent::UnarchiveProject(unarchive.clone()))
                        })),
                    )
                    .child(
                        secondary_button(
                            SharedString::from(format!("delete-project-{ix}")),
                            crate::i18n::text("settings-delete"),
                            true,
                            false,
                        )
                        .child(crate::i18n::text("settings-delete"))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(SettingsEvent::DeleteProject(delete.clone()))
                        })),
                    )
                    .into_any_element()
            })
            .collect();
        if projects.is_empty() {
            projects.push(note(crate::i18n::text("settings-no-archived-projects")));
        }
        vec![
            self.group(
                None,
                true,
                crate::i18n::text("settings-archived-projects"),
                Some(crate::i18n::text("settings-archive-a-project-from-the-rail-to").into()),
                None,
                projects,
            ),
            self.group(None, false, SharedString::from(title), None, None, rows),
        ]
    }
}

fn archive_date(updated_at: u64) -> String {
    chrono::DateTime::from_timestamp_millis(updated_at.min(i64::MAX as u64) as i64)
        .filter(|_| updated_at > 0)
        .map(|d| crate::system_format::date(d.timestamp_millis(), "MMMd", "%b %-d"))
        .unwrap_or_default()
}

#[cfg(all(test, target_os = "macos"))]
#[test]
fn archive_timestamp_date_follows_local_day() {
    for at in [1767310200000_u64, 1767313800000] {
        assert_eq!(
            archive_date(at),
            crate::system_format::local_date_reference(at as i64, "MMMd")
        );
    }
    assert_eq!(archive_date(0), "");
}
