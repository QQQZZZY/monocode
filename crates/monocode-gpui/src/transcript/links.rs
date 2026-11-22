//! AgentMarkdown's file-link actions, with browser actions for web links.
use super::*;
use crate::popover::{Menu, MenuAction, MenuEvent, MenuItem, menu_popover};
use gpui::{AnyElement, Entity, Point, Subscription};
use monocode_core::files;
use std::path::Path;

#[derive(Default)]
pub(super) struct State {
    pub(super) open: Option<OpenMenu>,
    error: Option<String>,
}
pub(super) struct OpenMenu {
    position: Point<Pixels>,
    menu: Entity<Menu>,
    _events: Subscription,
}
enum Target {
    File {
        path: String,
        location: Option<(usize, usize)>,
        base: Option<String>,
    },
    Web(String),
}
impl Target {
    fn resolve(url: &str, base: Option<&str>) -> Option<Self> {
        let lower = url.to_ascii_lowercase();
        if lower.starts_with("https://")
            || lower.starts_with("http://")
            || lower.starts_with("mailto:")
        {
            return Some(Self::Web(url.into()));
        }
        if url.is_empty()
            || url.starts_with(['#', '?'])
            || (url.contains("://") && !url.starts_with("file://"))
        {
            return None;
        }
        let (path, location) = crate::file_editor::file_location(url, base.unwrap_or(""));
        if !Path::new(&path).is_absolute() {
            return None;
        }
        Some(Self::File {
            path,
            location,
            base: base.map(str::to_owned),
        })
    }
    fn items(&self) -> Vec<MenuItem> {
        match self {
            Self::Web(_) => vec![
                MenuAction::new("open-web", crate::i18n::text("transcript-open-link")).into(),
                MenuAction::new(
                    "copy-link",
                    crate::i18n::text("transcript-copy-link-address"),
                )
                .into(),
            ],
            Self::File { base, .. } => {
                let reveal = if cfg!(target_os = "macos") {
                    crate::i18n::text("transcript-reveal-in-finder")
                } else if cfg!(target_os = "windows") {
                    crate::i18n::text("transcript-reveal-in-file-explorer")
                } else {
                    crate::i18n::text("transcript-open-containing-folder")
                };
                let mut items = vec![
                    MenuAction::new(
                        "open-monocode",
                        crate::i18n::text("transcript-open-in-monocode"),
                    )
                    .into(),
                    MenuAction::new(
                        "open-default",
                        crate::i18n::text("transcript-open-in-default-app"),
                    )
                    .into(),
                    MenuAction::new("reveal", reveal).into(),
                    MenuItem::Separator,
                    MenuAction::new("copy-path", crate::i18n::text("transcript-copy-path")).into(),
                ];
                if base.is_some() {
                    items.push(
                        MenuAction::new(
                            "copy-relative-path",
                            crate::i18n::text("transcript-copy-relative-path"),
                        )
                        .into(),
                    );
                }
                items
            }
        }
    }
}
impl TranscriptView {
    pub(super) fn open_link_menu(
        &mut self,
        request: &markdown::LinkMenu,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = Target::resolve(&request.url, self.file_link_base.as_deref()) else {
            return;
        };
        self.selection.hide_menu();
        self.links.error = None;
        self.links.open = None;
        // Replacing an open menu must restore to the transcript, rather than
        // retaining a focus handle owned by the menu that is being discarded.
        if let Some(focus) = &self.selection_focus {
            window.focus(focus, cx);
        }
        let label = if matches!(&target, Target::File { .. }) {
            crate::i18n::text("transcript-file-link-actions")
        } else {
            crate::i18n::text("transcript-link-actions")
        };
        let menu = cx.new(|cx| Menu::new(label, target.items(), window, cx));
        let events = cx.subscribe_in(
            &menu,
            window,
            move |this, _, event: &MenuEvent, window, cx| {
                this.links.open = None;
                if let MenuEvent::Picked(id, _) = event {
                    this.run_link_action(id, &target, window, cx);
                }
                cx.notify();
            },
        );
        self.links.open = Some(OpenMenu {
            position: request.position,
            menu,
            _events: events,
        });
        cx.stop_propagation();
        cx.notify();
    }
    fn run_link_action(
        &mut self,
        action: &str,
        target: &Target,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match (action, target) {
            ("open-web", Target::Web(url)) => cx.open_url(url),
            ("copy-link", Target::Web(url)) => {
                cx.write_to_clipboard(ClipboardItem::new_string(url.clone()))
            }
            ("open-monocode", Target::File { path, location, .. }) => {
                let target = location.map_or_else(
                    || path.clone(),
                    |(line, column)| format!("{path}:{line}:{column}"),
                );
                window.dispatch_action(Box::new(crate::file_editor::OpenPath(target)), cx);
            }
            ("copy-path", Target::File { path, .. }) => {
                cx.write_to_clipboard(ClipboardItem::new_string(path.clone()))
            }
            ("copy-relative-path", Target::File { path, base, .. }) => cx.write_to_clipboard(
                ClipboardItem::new_string(files::display_path(path, base.as_deref())),
            ),
            ("open-default" | "reveal", Target::File { path, .. }) => {
                let path = path.clone();
                let external = action == "open-default";
                let work = cx.background_executor().spawn(async move {
                    if external {
                        files::open_path_with_default_app(Path::new(&path))
                    } else {
                        files::reveal_path(Path::new(&path))
                    }
                });
                cx.spawn(async move |this, cx| {
                    if let Err(error) = work.await {
                        let _ = this.update(cx, |this, cx| {
                            let action = if external {
                                crate::i18n::text("transcript-open-file-default-action")
                            } else {
                                crate::i18n::text("transcript-complete-file-action")
                            };
                            this.links.error = Some(crate::i18n::format(
                                "transcript-could-not",
                                &[
                                    ("action", (action).to_string()),
                                    ("error", (error).to_string()),
                                ],
                            ));
                            cx.notify();
                        });
                    }
                })
                .detach();
            }
            _ => {}
        }
    }
    pub(super) fn link_overlays(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .absolute()
            .inset_0()
            .when_some(self.links.open.as_ref(), |d, open| {
                d.child(
                    menu_popover(&open.menu, Anchor::Point(open.position), cx)
                        .allow_outside_press(),
                )
            })
            .when_some(self.links.error.as_ref(), |d, error| {
                d.child(
                    div()
                        .absolute()
                        .top(px(12.))
                        .left_0()
                        .w_full()
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .w_full()
                                .max_w(px(COLUMN))
                                .px(px(10.))
                                .flex()
                                .justify_end()
                                .child(
                                    div().w(px(360.)).max_w_full().child(
                                        crate::notifications::errors::card(
                                            "transcript-link-error",
                                            error.clone(),
                                            icon_button(
                                                "dismiss-link-error",
                                                crate::i18n::text(
                                                    "transcript-dismiss-file-action-error",
                                                ),
                                                "close",
                                            )
                                            .debug_selector(|| "dismiss-link-error".into())
                                            .on_click(
                                                cx.listener(|this, _, _, cx| {
                                                    this.links.error = None;
                                                    cx.notify();
                                                }),
                                            ),
                                        ),
                                    ),
                                ),
                        ),
                )
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, VisualTestContext};

    #[gpui::test]
    fn file_action_error_fits_chat_and_dismisses(cx: &mut TestAppContext) {
        let window = cx.update(|cx| {
            cx.open_window(Default::default(), |_, cx| {
                cx.new(|_| TranscriptView::new())
            })
            .unwrap()
        });
        let visual = &mut VisualTestContext::from_window(window.into(), cx);
        for width in [320., 1280., 240.] {
            let message = format!(
                "Could not open file in default app: /fixture/{}END",
                "unbrokenpath".repeat(60)
            );
            visual.simulate_resize(gpui::size(px(width), px(600.)));
            window
                .update(visual, |view, _, cx| {
                    view.links.error = Some(message.clone());
                    cx.notify();
                })
                .unwrap();
            visual.run_until_parked();
            visual.refresh().unwrap();
            let card = visual
                .debug_bounds("transcript-link-error")
                .expect("error card");
            let close = visual.debug_bounds("dismiss-link-error").expect("close");
            let viewport = visual
                .debug_bounds("transcript-link-error-message-viewport")
                .expect("message viewport");
            let text = visual
                .debug_bounds("transcript-link-error-message")
                .expect("message");
            eprintln!(
                "width={width}, card={card:?}, close={close:?}, viewport={viewport:?}, text={text:?}"
            );
            assert!(
                card.left() >= px(0.) && card.right() <= px(width),
                "card must fit chat"
            );
            assert_eq!(card.right(), px((width + width.min(COLUMN)) / 2. - 10.));
            assert_eq!(card.top(), px(12.));
            assert!(
                close.left() >= card.left()
                    && close.right() <= card.right()
                    && close.top() >= card.top()
                    && close.bottom() <= card.bottom(),
                "close must fit card"
            );
            assert_eq!(close.size, gpui::size(px(24.), px(24.)));
            assert!(
                viewport.left() >= card.left()
                    && viewport.right() <= card.right()
                    && viewport.top() >= card.top()
                    && viewport.bottom() <= card.bottom()
            );
            assert!(text.left() >= viewport.left() && text.right() <= viewport.right());
            assert!(text.size.height > px(39.), "unbroken error must wrap");
            assert!(
                text.bottom() > viewport.bottom(),
                "long error must remain scrollable"
            );
            visual.simulate_event(gpui::ScrollWheelEvent {
                position: viewport.center(),
                delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-10000.))),
                modifiers: Default::default(),
                touch_phase: gpui::TouchPhase::Moved,
            });
            visual.run_until_parked();
            visual.refresh().unwrap();
            let scrolled = visual
                .debug_bounds("transcript-link-error-message")
                .expect("scrolled message");
            assert!(
                scrolled.top() < text.top() && scrolled.bottom() <= viewport.bottom() + px(1.),
                "full error must be reachable"
            );
            assert_eq!(
                window
                    .update(visual, |view, _, _| view.links.error.clone())
                    .unwrap(),
                Some(message)
            );
            visual.simulate_click(close.center(), Default::default());
            visual.run_until_parked();
            visual.refresh().unwrap();
            assert!(visual.debug_bounds("transcript-link-error").is_none());
            assert!(
                window
                    .update(visual, |view, _, _| view.links.error.is_none())
                    .unwrap()
            );
        }
    }

    #[gpui::test]
    fn file_action_error_aligns_in_both_conversation_layouts(cx: &mut TestAppContext) {
        use crate::conversation::Conversation;
        use monocode_core::store::Store;
        use std::sync::{Arc, Mutex};

        for mono in [false, true] {
            let store = Store::open(Path::new(":memory:")).unwrap();
            let mut summary = store.create("codex", "/fixture").unwrap();
            summary.has_messages = true;
            let window = cx.update(|cx| {
                crate::input::bind(cx);
                cx.open_window(Default::default(), |_, cx| {
                    cx.new(|cx| {
                        let mut view = Conversation::new(summary, Arc::new(Mutex::new(store)), cx);
                        if mono {
                            view.configure_mono(
                                monocode_core::mono::Mono::default(),
                                "/fixture".into(),
                                Default::default(),
                                cx,
                            );
                        }
                        view.transcript.update(cx, |view, cx| {
                            view.links.error = Some(format!(
                                "Could not open file: /fixture/{}",
                                "longpath".repeat(80)
                            ));
                            cx.notify();
                        });
                        view
                    })
                })
                .unwrap()
            });
            let visual = &mut VisualTestContext::from_window(window.into(), cx);
            for width in [360., 1400.] {
                visual.simulate_resize(gpui::size(px(width), px(800.)));
                visual.run_until_parked();
                visual.refresh().unwrap();
                let card = visual
                    .debug_bounds("transcript-link-error")
                    .expect("error card");
                let composer = visual
                    .debug_bounds(if mono {
                        "mono-composer"
                    } else {
                        "composer-box"
                    })
                    .expect("composer");
                let close = visual.debug_bounds("dismiss-link-error").expect("close");
                eprintln!(
                    "mono={mono}, width={width}, card={card:?}, composer={composer:?}, close={close:?}"
                );
                assert_eq!(card.right(), composer.right());
                assert!(
                    card.bottom() <= composer.top(),
                    "error must not cover composer"
                );
                assert!(close.right() <= card.right() && close.bottom() <= card.bottom());
            }
            let close = visual.debug_bounds("dismiss-link-error").unwrap();
            visual.simulate_click(close.center(), Default::default());
            visual.run_until_parked();
            visual.refresh().unwrap();
            assert!(visual.debug_bounds("transcript-link-error").is_none());
        }
    }
}
