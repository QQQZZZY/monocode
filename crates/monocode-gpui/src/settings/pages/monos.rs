use crate::settings::SettingsView;
use crate::{
    settings::{SettingsEvent, management::Target, widgets::icon_action},
    theme::*,
};
use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};
impl SettingsView {
    pub(super) fn monos_page(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let p = self.prefs.clone();
        let mut rows = vec![
            self.row(
                Some("monos-enabled"),
                true,
                crate::i18n::text("settings-show-monos"),
                Some(crate::i18n::text("settings-agents-of-your-own-on-the-project").into()),
                vec![self.toggle(
                    "monos-enabled",
                    crate::i18n::text("settings-show-monos"),
                    p.monos,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.monos = on),
                )],
            ),
            self.row(
                Some("rail-monos-pinned"),
                false,
                crate::i18n::text("settings-pin-monos-to-the-icon-rail"),
                Some(crate::i18n::text("settings-when-the-project-rail-is-collapsed-to").into()),
                vec![self.toggle(
                    "rail-monos-pinned",
                    crate::i18n::text("settings-pin-monos-to-the-icon-rail"),
                    p.rail_monos_pinned,
                    !p.monos,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.rail_monos_pinned = on),
                )],
            ),
        ];
        if cfg!(target_os = "macos") {
            rows.push(self.row(
                Some("mono-menu-bar-icon"),
                false,
                crate::i18n::text("settings-menu-bar-icon"),
                Some(crate::i18n::text("settings-chat-with-a-mono-or-open-the").into()),
                vec![self.toggle(
                    "mono-menu-bar-icon",
                    crate::i18n::text("settings-menu-bar-icon"),
                    p.mono_menu_bar_icon,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.mono_menu_bar_icon = on),
                )],
            ));
        }
        let mut roster = Vec::new();
        for (ix, mono) in self.ctx.monos.clone().into_iter().enumerate() {
            let show_id = mono.id.clone();
            let show = mono.show;
            let folders = mono.folders;
            let description = match mono.projects.split_last() {
                None => crate::i18n::text("settings-no-projects-yet"),
                Some((last, [])) => {
                    crate::i18n::format("settings-works-on", &[("last", (last).to_string())])
                }
                Some((last, others)) => crate::i18n::format(
                    "settings-works-on-and",
                    &[
                        ("value", (others.join(", ")).to_string()),
                        ("last", (last).to_string()),
                    ],
                ),
            };
            let label = div()
                .min_w_0()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .debug_selector(|| "settings-mono-mascot".into())
                        .size(px(16.))
                        .flex_shrink_0()
                        .child(crate::monos::avatar::avatar(
                            &mono.mascot,
                            mono.color,
                            "idle",
                            16.,
                            true,
                        )),
                )
                .child(
                    div()
                        .debug_selector(|| "settings-mono-name".into())
                        .min_w_0()
                        .truncate()
                        .child(mono.name.clone()),
                );
            let reset = mono.clone();
            let controls = vec![
                div()
                    .text_size(px(12.))
                    .line_height(px(20.))
                    .text_color(content(0.5))
                    .child(crate::i18n::text(
                        "settings-show-mono-spawned-session-on-the",
                    ))
                    .into_any_element(),
                div()
                    .debug_selector(|| "settings-mono-sidebar-switch".into())
                    .flex_shrink_0()
                    .child(self.toggle(
                        "mono-sidebar-show",
                        crate::i18n::format(
                            "settings-show-sessions-started-by-in-sidebar",
                            &[("value", (mono.name).to_string())],
                        ),
                        show,
                        false,
                        cx,
                        move |_, on, cx| {
                            cx.emit(SettingsEvent::MonoSidebar(show_id.clone(), on, folders))
                        },
                    ))
                    .into_any_element(),
                icon_action(
                    "reset-mono",
                    crate::i18n::format(
                        "settings-reset-to-defaults",
                        &[("value", (mono.name).to_string())],
                    ),
                    "rotate-ccw",
                    28.,
                    14.,
                    0.4,
                    false,
                )
                .debug_selector(|| "settings-mono-reset".into())
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.open_management(Target::ResetMono(reset.clone()), window, cx)
                }))
                .into_any_element(),
            ];
            roster.push(
                div()
                    .id(SharedString::from(format!("settings-mono-{}", mono.id)))
                    .child(self.row(None, ix == 0, label, Some(description.into()), controls))
                    .into_any_element(),
            );
        }
        if roster.is_empty() {
            roster.push(super::super::widgets::note(crate::i18n::text(
                "settings-no-monos-yet",
            )));
        }
        vec![
            self.group(
                None,
                true,
                crate::i18n::text("settings-monos"),
                None,
                None,
                rows,
            ),
            self.group(
                Some("mono-list"),
                false,
                crate::i18n::text("settings-your-monos"),
                Some(crate::i18n::text("settings-show-or-hide-sessions-started-by-each").into()),
                None,
                roster,
            ),
        ]
    }
}
