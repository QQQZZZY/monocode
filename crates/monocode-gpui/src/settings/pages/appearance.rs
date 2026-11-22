//! AppearancePage, ChatBackgroundCard, workspace themes and AccentColorPicker.
use crate::popover::{Align, Anchor, Side, popover};
use crate::settings::{
    SettingsEvent, SettingsView, SliderKind, preferences::UI_SCALE_PERCENTS,
    widgets::secondary_button,
};
use crate::theme::*;
use gpui::{
    AnyElement, Context, Focusable, Role, SharedString, Window, div, linear_color_stop,
    linear_gradient, prelude::*, px, rgb,
};

/// ACCENT_COLOR_PRESETS, after the Default swatch.
pub(crate) const ACCENTS: [(u32, &str); 6] = [
    (0x4da3f5, "settings-blue"),
    (0x8b5cf6, "settings-violet"),
    (0xec4899, "settings-pink"),
    (0xef4444, "settings-red"),
    (0xf59e0b, "settings-orange"),
    (0x10b981, "settings-green"),
];

const EFFECTS: [(&str, &str); 6] = [
    ("none", "settings-none"),
    ("dither", "settings-dither"),
    ("ascii", "settings-ascii"),
    ("halftone", "settings-halftone"),
    ("scanlines", "settings-scanlines"),
    ("gradient-blur", "settings-haze"),
];
fn effect_description(effect: &str) -> String {
    match effect {
        "dither" => crate::i18n::text("settings-rebuilds-the-artwork-with-a-dithered"),
        "ascii" => crate::i18n::text("settings-recreates-the-artwork-with-colored-characters"),
        "halftone" => crate::i18n::text("settings-recreates-the-artwork-with-colored-print"),
        "scanlines" => crate::i18n::text("settings-adds-a-pronounced-horizontal-display-line"),
        "gradient-blur" => crate::i18n::text("settings-blurs-and-fades-the-artwork-into"),
        _ => crate::i18n::text("settings-shows-the-original-artwork"),
    }
}

pub(crate) fn parse_hex(text: &str) -> Option<u32> {
    let hex = text.strip_prefix('#')?;
    (hex.len() == 6)
        .then(|| u32::from_str_radix(hex, 16).ok())
        .flatten()
}

/// Preview the palette's neutral tones and accent at a menu icon's 16px.
pub(crate) fn swatch(surface: u32, panel: u32, solid: u32, accent: u32) -> AnyElement {
    div()
        .relative()
        .size_4()
        .overflow_hidden()
        .rounded(px(4.))
        .border_1()
        .border_color(content(0.15))
        .bg(rgb(surface))
        .child(
            div()
                .absolute()
                .top(px(2.))
                .bottom(px(2.))
                .left(px(4.))
                .right(px(-1.))
                .rounded_l(px(2.))
                .border_1()
                .border_color(rgb(solid))
                .bg(rgb(panel)),
        )
        .child(
            div()
                .absolute()
                .top(px(5.))
                .left(px(7.))
                .size(px(5.))
                .rounded_full()
                .bg(rgb(accent)),
        )
        .into_any_element()
}

impl SettingsView {
    pub(super) fn appearance_page(
        &mut self,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let p = self.prefs.clone();
        let inset = !self.ctx.classic;
        let light = p.theme == "light"
            || (p.theme == "system"
                && matches!(
                    cx.window_appearance(),
                    gpui::WindowAppearance::Light | gpui::WindowAppearance::VibrantLight
                ));
        let mut theme = vec![self.row(
            Some("theme"),
            true,
            crate::i18n::text("settings-theme"),
            Some(crate::i18n::text("settings-system-follows-the-os-appearance").into()),
            vec![self.segmented(
                "theme",
                crate::i18n::text("settings-theme"),
                &p.theme,
                &[
                    ("system", &crate::i18n::text("settings-system")),
                    ("dark", &crate::i18n::text("settings-dark")),
                    ("light", &crate::i18n::text("settings-light")),
                ],
                cx,
                |this, v, cx| this.update(cx, |p| p.theme = v.into()),
            )],
        )];
        let options = crate::settings::palettes::PALETTES
            .iter()
            .map(|&(id, label, dark, pale)| {
                let palette = if light { pale } else { dark };
                (
                    id.to_owned(),
                    SharedString::from(if id == "default" {
                        crate::i18n::text("settings-default")
                    } else {
                        label.to_owned()
                    }),
                    Some(swatch(
                        palette.surface,
                        palette.panel,
                        palette.solid,
                        palette.accent,
                    )),
                )
            })
            .collect();
        theme.push(self.row(
            Some("workspace-theme"),
            false,
            crate::i18n::text("settings-workspace-theme"),
            Some(crate::i18n::text("settings-choose-a-color-preset").into()),
            vec![self.select(
                "workspace-theme",
                crate::i18n::text("settings-workspace-theme"),
                &p.workspace_theme,
                options,
                Some(192.),
                cx,
                |this, v, cx| this.update(cx, |p| p.apply_workspace_theme(v)),
            )],
        ));
        #[cfg(target_os = "linux")]
        if cx.compositor_name() == "Wayland" {
            theme.push(
                self.row(
                    Some("wayland-window-decorations"),
                    false,
                    crate::i18n::text("settings-wayland-window-decorations"),
                    Some(
                        crate::i18n::text(
                            "settings-show-compositor-decorations-monocode-s-title-bar",
                        )
                        .into(),
                    ),
                    vec![self.toggle(
                        "wayland-window-decorations",
                        crate::i18n::text("settings-wayland-window-decorations"),
                        p.wayland_window_decorations,
                        false,
                        cx,
                        |this, on, cx| this.update(cx, |p| p.wayland_window_decorations = on),
                    )],
                ),
            );
        }
        let accent_picker = self.accent_picker(cx);
        theme.push(self.row(
            Some("accent-color"),
            false,
            crate::i18n::text("settings-accent-color"),
            Some(crate::i18n::text("settings-used-for-the-composer-send-button-and").into()),
            vec![accent_picker],
        ));
        theme.push(self.row(
            Some("diff-colors"),
            false,
            crate::i18n::text("settings-diff-colors"),
            Some(crate::i18n::text("settings-colorblind-and-high-contrast-use-blue-and").into()),
            vec![self.segmented(
                "diff-colors",
                crate::i18n::text("settings-diff-colors"),
                &p.diff_palette,
                &[
                    ("default", &crate::i18n::text("settings-default")),
                    ("colorblind", &crate::i18n::text("settings-colorblind")),
                    (
                        "high-contrast",
                        &crate::i18n::text("settings-high-contrast"),
                    ),
                ],
                cx,
                |this, v, cx| this.update(cx, |p| p.diff_palette = v.into()),
            )],
        ));
        let mut page = vec![self.group(
            None,
            true,
            crate::i18n::text("settings-theme"),
            None,
            None,
            theme,
        )];
        let color = vec![
            self.row(
                Some("hue"),
                true,
                crate::i18n::text("settings-hue"),
                Some(crate::i18n::text("settings-base-hue-for-the-window-and-main").into()),
                vec![self.slider(SliderKind::Hue, format!("{}°", p.theme_hue), false, cx)],
            ),
            self.row(
                Some("saturation"),
                false,
                crate::i18n::text("settings-saturation"),
                Some(crate::i18n::text("settings-how-strongly-the-hue-tints-the-interface").into()),
                vec![self.slider(
                    SliderKind::Saturation,
                    crate::system_format::percent(p.theme_saturation as f64),
                    false,
                    cx,
                )],
            ),
            self.row(
                Some("dark-lightness"),
                false,
                crate::i18n::text("settings-dark-mode-lightness"),
                Some(if light {
                    crate::i18n::text("settings-this-only-affects-dark-mode-your-dark-mode").into()
                } else {
                    crate::i18n::text("settings-base-brightness-of-the-dark-theme-lower-values")
                        .into()
                }),
                vec![self.slider(
                    SliderKind::DarkLightness,
                    crate::system_format::percent(p.theme_dark_lightness as f64),
                    light,
                    cx,
                )],
            ),
        ];
        page.push(self.group(
            None,
            false,
            crate::i18n::text("settings-color"),
            None,
            None,
            color,
        ));
        let translucency = vec![
            self.row(
                Some("sidebar-opacity"),
                true,
                crate::i18n::text("settings-sidebar-opacity"),
                Some(crate::i18n::text("settings-applies-to-the-project-rail-and-the").into()),
                vec![self.slider(
                    SliderKind::SidebarOpacity,
                    crate::system_format::percent(p.sidebar_opacity as f64),
                    light,
                    cx,
                )],
            ),
            self.row(
                Some("main-pane-glass"),
                false,
                crate::i18n::text("settings-main-pane-glass"),
                Some(
                    crate::i18n::text("settings-extend-the-translucent-treatment-to-the-main")
                        .into(),
                ),
                vec![self.toggle(
                    "main-pane-glass",
                    crate::i18n::text("settings-main-pane-glass"),
                    p.body_glass,
                    light,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.body_glass = on),
                )],
            ),
        ];
        page.push(self.group(
            None,
            false,
            crate::i18n::text("settings-translucency"),
            Some(if light {
                crate::i18n::text("settings-translucency-is-available-in-dark-mode").into()
            } else {
                crate::i18n::text("settings-how-much-of-the-desktop-shows-through-monocode").into()
            }),
            None,
            translucency,
        ));
        page.push(self.chat_background_card(cx));
        let scale_options = UI_SCALE_PERCENTS
            .iter()
            .map(|percent| {
                (
                    percent.to_string(),
                    SharedString::from(crate::system_format::percent(*percent as f64)),
                    None,
                )
            })
            .collect();
        let mut layout = vec![
            self.row(
                Some("inset-layout"),
                true,
                crate::i18n::text("settings-layout"),
                Some(
                    crate::i18n::text("settings-classic-keeps-everything-in-one-sidebar-focused")
                        .into(),
                ),
                vec![self.segmented(
                    "inset-layout",
                    crate::i18n::text("settings-layout"),
                    if inset { "focused" } else { "classic" },
                    &[
                        ("classic", &crate::i18n::text("settings-classic")),
                        ("focused", &crate::i18n::text("settings-focused")),
                    ],
                    cx,
                    |_, value, cx| cx.emit(SettingsEvent::NewLayout(value == "focused")),
                )],
            ),
            self.row(
                Some("interface-scale"),
                false,
                crate::i18n::text("settings-interface-scale"),
                Some(crate::i18n::text("settings-zoom-the-whole-interface-you-can-also").into()),
                vec![self.select(
                    "interface-scale",
                    crate::i18n::text("settings-interface-scale"),
                    &p.ui_scale.to_string(),
                    scale_options,
                    None,
                    cx,
                    |this, v, cx| {
                        if let Ok(v) = v.parse() {
                            this.update(cx, |p| p.ui_scale = v)
                        }
                    },
                )],
            ),
            self.row(
                Some("show-excluded-files"),
                false,
                crate::i18n::text("settings-show-excluded-files"),
                Some(crate::i18n::text("settings-show-files-and-folders-git-excludes-such").into()),
                vec![self.toggle(
                    "show-excluded-files",
                    crate::i18n::text("settings-show-excluded-files"),
                    p.show_excluded_files,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.show_excluded_files = on),
                )],
            ),
        ];
        if inset {
            layout.insert(
                1,
                self.row(
                    Some("new-layout-style"),
                    false,
                    crate::i18n::text("settings-workspace-style"),
                    Some(if cfg!(target_os = "macos") {
                        crate::i18n::text("settings-choose-a-rounded-card-or-a-full-height").into()
                    } else {
                        crate::i18n::text("settings-choose-a-rounded-card-or-a-full-height-2")
                            .into()
                    }),
                    vec![self.segmented(
                        "new-layout-style",
                        crate::i18n::text("settings-workspace-style"),
                        if self.ctx.new_layout_boxy {
                            "boxy"
                        } else {
                            "rounded"
                        },
                        &[
                            ("rounded", &crate::i18n::text("settings-rounded")),
                            ("boxy", &crate::i18n::text("settings-boxy")),
                        ],
                        cx,
                        |_, value, cx| cx.emit(SettingsEvent::NewLayoutBoxy(value == "boxy")),
                    )],
                ),
            );
        }
        page.push(self.group(
            None,
            false,
            crate::i18n::text("settings-layout"),
            None,
            None,
            layout,
        ));
        page
    }

    /// AccentColorPicker: Default plus six presets and a custom color.
    fn accent_picker(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let value = self.prefs.accent_color.clone();
        let preset = match &value {
            None => Some(0),
            Some(hex) => ACCENTS
                .iter()
                .position(|(c, _)| Some(*c) == parse_hex(hex))
                .map(|ix| ix + 1),
        };
        let custom = preset
            .is_none()
            .then(|| value.as_deref().and_then(parse_hex))
            .flatten();
        let weak = cx.weak_entity();
        let dot = |selected: bool, color: gpui::Hsla| {
            div()
                .size(px(14.))
                .rounded_full()
                .bg(color)
                .when(selected, |d| d.border_2().border_color(content(0.8)))
        };
        let colors: Vec<(gpui::Hsla, &'static str, Option<String>)> =
            std::iter::once((content(1.), "settings-default", None))
                .chain(
                    ACCENTS
                        .iter()
                        .map(|&(c, label)| (rgb(c).into(), label, Some(format!("#{c:06x}")))),
                )
                .collect();
        div()
            .w(px(192.))
            .relative()
            .child(self.accent_anchor.probe())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_1()
                    .px(px(2.))
                    .children(
                        colors
                            .into_iter()
                            .enumerate()
                            .map(|(ix, (color, label, hex))| {
                                div()
                                    .id(("accent-color", ix))
                                    .debug_selector(move || format!("accent-color-{ix}"))
                                    .role(Role::Button)
                                    .aria_label(crate::i18n::text(label))
                                    .aria_selected(preset == Some(ix))
                                    .size(px(20.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .cursor_pointer()
                                    .child(dot(preset == Some(ix), color))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.accent_custom_open = false;
                                        let hex = hex.clone();
                                        this.update(cx, |p| p.accent_color = hex);
                                    }))
                            }),
                    )
                    .child(
                        div()
                            .id("accent-custom")
                            .role(Role::Button)
                            .aria_label(crate::i18n::text("settings-custom-color"))
                            .aria_expanded(self.accent_custom_open)
                            .size(px(20.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_full()
                            .cursor_pointer()
                            .child(
                                div()
                                    .size(px(14.))
                                    .rounded_full()
                                    .overflow_hidden()
                                    .when(custom.is_some() || self.accent_custom_open, |d| {
                                        d.border_2().border_color(content(0.8))
                                    })
                                    .map(|d| match custom {
                                        Some(c) => d.bg(rgb(c)),
                                        None => d.bg(linear_gradient(
                                            135.,
                                            linear_color_stop(rgb(0xff4d4d), 0.),
                                            linear_color_stop(rgb(0x4d6bff), 1.),
                                        )),
                                    }),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.accent_custom_open = !this.accent_custom_open;
                                if this.accent_custom_open {
                                    let text = this
                                        .prefs
                                        .accent_color
                                        .clone()
                                        .unwrap_or_else(|| "#4da3f5".into());
                                    this.accent_custom.update(cx, |i, cx| {
                                        i.set_text(text, cx);
                                        i.select_range(0..i.content.len(), cx);
                                    });
                                    window.focus(&this.accent_custom.focus_handle(cx), cx);
                                }
                                cx.notify();
                            })),
                    ),
            )
            .when(self.accent_custom_open, |d| {
                d.child(
                    popover(
                        "accent-custom-popover",
                        Anchor::Slot(self.accent_anchor.clone()),
                    )
                    .side(Side::Bottom)
                    .align(Align::End)
                    .width(248.)
                    .padding(8.)
                    .on_dismiss(move |_, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            this.accent_custom_open = false;
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .h(px(30.))
                            .px_2()
                            .flex()
                            .items_center()
                            .gap_2()
                            .rounded_md()
                            .border_1()
                            .border_color(content(0.1))
                            .child(
                                div()
                                    .size(px(14.))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .bg(rgb(custom.unwrap_or(0x4da3f5))),
                            )
                            .child(div().flex_1().min_w_0().child(self.accent_custom.clone())),
                    ),
                )
            })
            .into_any_element()
    }

    /// ChatBackgroundCard: the global image behind every chat pane.
    fn chat_background_card(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let p = self.prefs.clone();
        let image = p
            .chat_background
            .clone()
            .filter(|path| std::path::Path::new(path).is_file());
        let empty = p.chat_background_empty;
        let session = p.chat_background_session;
        let preview = div()
            .p_4()
            .child(
                div()
                    .overflow_hidden()
                    .rounded_lg()
                    .border_1()
                    .border_color(content(0.1))
                    .map(|d| match &image {
                        Some(path) => d.child(
                            div()
                                .relative()
                                .h(px(144.))
                                .child(
                                    crate::image_memory::preview(std::path::PathBuf::from(path))
                                        .size_full()
                                        .object_fit(gpui::ObjectFit::Cover)
                                        .opacity(empty as f32 / 100.),
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .bottom_2()
                                        .left_2()
                                        .text_size(px(11.))
                                        .text_color(content(0.4))
                                        .child(crate::i18n::format(
                                            "settings-empty-chat-preview-at",
                                            &[(
                                                "empty",
                                                crate::system_format::percent(empty as f64),
                                            )],
                                        )),
                                ),
                        ),
                        None => d.child(
                            div()
                                .id("choose-chat-background")
                                .role(Role::Button)
                                .aria_label(crate::i18n::text("settings-choose-an-image"))
                                .h(px(144.))
                                .w_full()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_color(content(0.4))
                                .cursor_pointer()
                                .hover(|s| s.bg(content(0.05)).text_color(content(0.7)))
                                .child(icon("image-add", 20.).text_color(content(0.4)))
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .child(crate::i18n::text("settings-choose-an-image")),
                                )
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.emit(SettingsEvent::ChooseBackground)
                                })),
                        ),
                    }),
            )
            .when(image.is_some(), |d| {
                d.child(
                    div()
                        .mt_3()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2()
                        .child(
                            secondary_button(
                                "change-chat-background",
                                crate::i18n::text("settings-change"),
                                false,
                                false,
                            )
                            .child(crate::i18n::text("settings-change"))
                            .on_click(
                                cx.listener(|_, _, _, cx| cx.emit(SettingsEvent::ChooseBackground)),
                            ),
                        )
                        .child(
                            secondary_button(
                                "remove-chat-background",
                                crate::i18n::text("settings-remove"),
                                true,
                                false,
                            )
                            .child(crate::i18n::text("settings-remove"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.update(cx, |p| p.chat_background = None)
                            })),
                        ),
                )
            })
            .into_any_element();
        let mut rows = vec![preview];
        if image.is_some() {
            rows.push(self.row(
                None,
                false,
                crate::i18n::text("settings-background-effect"),
                Some(effect_description(&p.chat_background_effect).into()),
                vec![
                        self.segmented(
                            "background-effect",
                            crate::i18n::text("settings-background-effect"),
                            &p.chat_background_effect,
                            &EFFECTS
                                .map(|(id, key)| (id, crate::i18n::text(key)))
                                .iter()
                                .map(|(id, label)| (*id, label.as_str()))
                                .collect::<Vec<_>>(),
                            cx,
                            |this, v, cx| this.update(cx, |p| p.chat_background_effect = v.into()),
                        ),
                    ],
            ));
            rows.push(self.row(
                None,
                false,
                crate::i18n::text("settings-show-on"),
                Some(
                    crate::i18n::text("settings-empty-sessions-only-or-every-conversation").into(),
                ),
                vec![self.segmented(
                    "background-scope",
                    crate::i18n::text("settings-show-background-on"),
                    &p.chat_background_scope,
                    &[
                        ("empty", &crate::i18n::text("settings-empty-only")),
                        ("all", &crate::i18n::text("settings-all-sessions")),
                    ],
                    cx,
                    |this, v, cx| this.update(cx, |p| p.chat_background_scope = v.into()),
                )],
            ));
            rows.push(
                self.row(
                    None,
                    false,
                    crate::i18n::text("settings-empty-chat-visibility"),
                    Some(
                        crate::i18n::text(
                            "settings-background-strength-before-a-chat-has-messages",
                        )
                        .into(),
                    ),
                    vec![self.slider(
                        SliderKind::BackgroundEmpty,
                        crate::system_format::percent(empty as f64),
                        false,
                        cx,
                    )],
                ),
            );
            rows.push(
                self.row(
                    None,
                    false,
                    crate::i18n::text("settings-session-visibility"),
                    Some(
                        crate::i18n::text(
                            "settings-background-strength-once-the-conversation-has-messages",
                        )
                        .into(),
                    ),
                    vec![self.slider(
                        SliderKind::BackgroundSession,
                        crate::system_format::percent(session as f64),
                        false,
                        cx,
                    )],
                ),
            );
        }
        self.group(
            Some("chat-background"),
            false,
            crate::i18n::text("settings-chat-background"),
            Some(crate::i18n::text("settings-an-image-behind-your-chat-panes-it").into()),
            None,
            rows,
        )
    }
}
