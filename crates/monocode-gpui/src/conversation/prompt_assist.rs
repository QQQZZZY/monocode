//! T3 Code's composer assists: the suggested next prompt (provider stream,
//! parked Claude/Codex harvest, then the HTTP text generator), Up/Down recall
//! of this session's prompts, and Cmd/Ctrl+S prompt stashes.
use super::follow_up::{message_label, queue_button};
use super::*;
use monocode_core::provider::PromptSuggestionRequest;

/// Exchanges handed to the HTTP text generator for a suggestion.
const SUGGESTION_EXCHANGES: usize = 3;

impl Conversation {
    /// Gives the composer this session's prompts for recall and its stash key.
    pub(super) fn install_prompt_assist(
        input: &Entity<input::Input>,
        transcript: &Entity<TranscriptView>,
        cx: &mut Context<Self>,
    ) -> Subscription {
        let weak = transcript.downgrade();
        let history: input::PromptHistory = std::rc::Rc::new(move |cx: &gpui::App| {
            weak.upgrade()
                .map(|view| view.read(cx).user_prompts())
                .unwrap_or_default()
        });
        input.update(cx, |input, _| {
            input.set_prompt_history(Some(history));
            input.set_stash_enabled(true);
        });
        cx.subscribe(input, |this, _, _: &input::StashRequested, cx| {
            this.stash_or_restore(cx)
        })
    }

    fn prompt_suggestions_enabled(&self, cx: &gpui::App) -> bool {
        self.settings.mono.is_none()
            && cx
                .try_global::<crate::settings::Preferences>()
                .is_none_or(|prefs| prefs.prompt_suggestions)
    }

    /// Drops any pending or shown suggestion, e.g. when a new turn starts.
    pub(super) fn clear_prompt_suggestion(&mut self, cx: &mut Context<Self>) {
        if let Some(cancel) = self.suggestion_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.input.update(cx, |input, cx| input.set_ghost(None, cx));
    }

    /// Shows a suggestion as ghost text while the composer is idle and empty.
    pub(super) fn show_prompt_suggestion(&mut self, text: String, cx: &mut Context<Self>) {
        if !self.prompt_suggestions_enabled(cx) || self.stopping {
            return;
        }
        if let Some(cancel) = self.suggestion_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.input.update(cx, |input, cx| {
            if input.content.is_empty() && !input.is_composing() {
                input.set_ghost(Some(text), cx);
            }
        });
    }

    /// After a turn without a streamed suggestion: ask the parked Claude/Codex
    /// process, then the HTTP text generator from Settings.
    pub(super) fn harvest_prompt_suggestion(
        &mut self,
        request: Option<PromptSuggestionRequest>,
        cx: &mut Context<Self>,
    ) {
        if !self.prompt_suggestions_enabled(cx) || self.is_remote() {
            return;
        }
        let http = cx
            .try_global::<crate::settings::Preferences>()
            .and_then(|prefs| prefs.http_text_generator());
        if request.is_none() && http.is_none() {
            return;
        }
        let exchanges = if http.is_some() {
            self.transcript
                .read(cx)
                .recent_exchanges(SUGGESTION_EXCHANGES)
        } else {
            Vec::new()
        };
        if let Some(cancel) = self.suggestion_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        let cancel = Arc::new(AtomicBool::new(false));
        self.suggestion_cancel = Some(cancel.clone());
        let flag = cancel.clone();
        let session = self.summary.id;
        let work = cx.background_executor().spawn(async move {
            // Unit tests never reach live providers or the network.
            if cfg!(test) {
                return None;
            }
            let harvested = request.and_then(|request| {
                monocode_core::process::owned_by(session, || {
                    monocode_core::provider::suggest_next_prompt(request, flag.clone())
                })
            });
            harvested.or_else(|| {
                http.and_then(|config| {
                    monocode_core::text_generator::suggest_next_prompt(&config, &exchanges, flag)
                })
            })
        });
        cx.spawn(async move |this, cx| {
            let suggestion = work.await;
            let _ = this.update(cx, |this, cx| {
                if cancel.load(Ordering::Relaxed) || this.busy {
                    return;
                }
                this.suggestion_cancel = None;
                if let Some(text) = suggestion {
                    this.show_prompt_suggestion(text, cx);
                }
            });
        })
        .detach();
    }

    /// Cmd/Ctrl+S: stash a non-empty draft, or restore the latest stash into an
    /// empty composer.
    pub(super) fn stash_or_restore(&mut self, cx: &mut Context<Self>) {
        if self.is_remote() || self.attaching || self.settings.mono.is_some() {
            return;
        }
        let text = self.input.read(cx).content.to_string();
        if !text.trim().is_empty() || !self.attachments.is_empty() {
            let attachments = std::mem::take(&mut self.attachments);
            let message = self.message(text, attachments, None, cx);
            self.follow_ups.state.stashes.push(message);
            self.input
                .update(cx, |input, cx| input.set_text(String::new(), cx));
            self.save_follow_ups(cx);
        } else if !self.follow_ups.state.stashes.is_empty() {
            let last = self.follow_ups.state.stashes.len() - 1;
            self.restore_stash(last, cx);
        }
    }

    fn restore_stash(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.follow_ups.state.stashes.len() {
            return;
        }
        let draft = self.input.read(cx).content.to_string();
        if !draft.trim().is_empty() || !self.attachments.is_empty() {
            // Never overwrite a draft: stash it first, then restore.
            let attachments = std::mem::take(&mut self.attachments);
            let message = self.message(draft, attachments, None, cx);
            self.follow_ups.state.stashes.insert(index, message);
            self.input
                .update(cx, |input, cx| input.set_text(String::new(), cx));
            return self.restore_stash(index + 1, cx);
        }
        let message = self.follow_ups.state.stashes.remove(index);
        self.attachments = message.attachments;
        self.input
            .update(cx, |input, cx| input.set_text(message.text, cx));
        if self.follow_ups.state.stashes.is_empty() {
            self.stashes_open = false;
        }
        self.save_follow_ups(cx);
    }

    fn remove_stash(&mut self, id: &str, cx: &mut Context<Self>) {
        self.follow_ups.state.stashes.retain(|stash| stash.id != id);
        if self.follow_ups.state.stashes.is_empty() {
            self.stashes_open = false;
        }
        self.save_follow_ups(cx);
    }

    /// The stash shelf above the composer: a count toggle and, when open, the
    /// stashed drafts newest first.
    pub(super) fn stash_shelf(&self, cx: &mut Context<Self>) -> AnyElement {
        let stashes = &self.follow_ups.state.stashes;
        if stashes.is_empty() || self.is_remote() {
            return div().into_any_element();
        }
        let shortcut = if cfg!(target_os = "macos") {
            "⌘"
        } else {
            "Ctrl+"
        };
        let count = crate::system_format::unsigned(stashes.len() as u64);
        let header = div()
            .id("stash-toggle")
            .role(Role::Button)
            .aria_label(crate::i18n::format(
                "conversation-stashes-tooltip",
                &[("shortcut", shortcut.to_string())],
            ))
            .h(px(28.))
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .text_color(content(0.6))
            .hover(|s| s.text_color(content(0.9)))
            .child(icon("queue-list", 14.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(crate::i18n::format(
                        "conversation-stashes-count",
                        &[("count", count.to_string())],
                    )),
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.stashes_open = !this.stashes_open;
                cx.notify();
            }));
        let card =
            composer_shelf::card("stash-card", self.classic_layout)
                .child(header)
                .when(self.stashes_open, |card| {
                    card.children(stashes.iter().enumerate().rev().map(|(index, stash)| {
                        let label = if stash.text.trim().is_empty() {
                            crate::i18n::format(
                                "conversation-stash-attachment-only",
                                &[(
                                    "count",
                                    crate::system_format::unsigned(stash.attachments.len() as u64),
                                )],
                            )
                        } else {
                            message_label(stash)
                        };
                        let attachments = (!stash.text.trim().is_empty()
                            && !stash.attachments.is_empty())
                        .then(|| {
                            crate::i18n::format(
                                "conversation-stash-attachment-count",
                                &[(
                                    "count",
                                    crate::system_format::unsigned(stash.attachments.len() as u64),
                                )],
                            )
                        });
                        let remove = stash.id.clone();
                        div()
                            .id(SharedString::from(format!("stash-{}", stash.id)))
                            .role(Role::Group)
                            .aria_label(label.clone())
                            .min_h(px(30.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .border_t_1()
                            .border_color(content(0.05))
                            .child(
                                div()
                                    .id("stash-restore")
                                    .role(Role::Button)
                                    .aria_label(crate::i18n::text("conversation-stash-restore"))
                                    .flex_1()
                                    .min_w_0()
                                    .py_1()
                                    .cursor_pointer()
                                    .text_color(content(0.75))
                                    .hover(|s| s.text_color(content(0.95)))
                                    .truncate()
                                    .child(label)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.restore_stash(index, cx)
                                    })),
                            )
                            .children(attachments.map(|text| {
                                div().flex_shrink_0().text_color(content(0.4)).child(text)
                            }))
                            .child(
                                queue_button(
                                    "stash-remove",
                                    crate::i18n::text("conversation-stash-remove"),
                                    "trash",
                                )
                                .on_click(
                                    cx.listener(move |this, _, _, cx| {
                                        this.remove_stash(&remove, cx)
                                    }),
                                ),
                            )
                    }))
                    .child(
                        div()
                            .id("stash-clear-all")
                            .role(Role::Button)
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .border_t_1()
                            .border_color(content(0.05))
                            .cursor_pointer()
                            .text_color(content(0.45))
                            .hover(|s| s.text_color(content(0.85)))
                            .child(crate::i18n::text("conversation-stash-clear-all"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.follow_ups.state.stashes.clear();
                                this.stashes_open = false;
                                this.save_follow_ups(cx);
                            })),
                    )
                });
        composer_shelf::shelf("stash-shelf")
            .child(card)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, WindowHandle};
    use monocode_core::store::Store;
    use std::path::PathBuf;
    use std::sync::Mutex;

    fn conversation(cx: &mut TestAppContext) -> WindowHandle<Conversation> {
        let store = Store::open(Path::new(":memory:")).unwrap();
        let summary = store.create("claude", "/tmp").unwrap();
        cx.update(|cx| {
            input::bind(cx);
            cx.set_global(crate::settings::Preferences::default());
            cx.open_window(Default::default(), |_, cx| {
                cx.new(|cx| {
                    let mut view = Conversation::new(summary, Arc::new(Mutex::new(store)), cx);
                    view.loaded = true;
                    view
                })
            })
            .unwrap()
        })
    }

    fn content(window: &WindowHandle<Conversation>, cx: &mut TestAppContext) -> String {
        window
            .update(cx, |view, _, cx| view.input.read(cx).content.to_string())
            .unwrap()
    }

    #[gpui::test]
    fn stash_saves_a_draft_and_restores_it_into_an_empty_composer(cx: &mut TestAppContext) {
        let window = conversation(cx);
        window
            .update(cx, |view, _, cx| {
                view.input.update(cx, |input, cx| {
                    input.set_text("Finish auth refactor".into(), cx)
                });
                view.attachments = vec![PathBuf::from("/tmp/stash-note.txt")];
                view.stash_or_restore(cx);
                assert_eq!(view.follow_ups.state.stashes.len(), 1);
                assert!(view.attachments.is_empty());
            })
            .unwrap();
        assert_eq!(content(&window, cx), "");
        window
            .update(cx, |view, _, cx| {
                view.stash_or_restore(cx);
                assert!(view.follow_ups.state.stashes.is_empty());
                assert_eq!(view.attachments, vec![PathBuf::from("/tmp/stash-note.txt")]);
            })
            .unwrap();
        assert_eq!(content(&window, cx), "Finish auth refactor");
    }

    #[gpui::test]
    fn restoring_from_the_shelf_keeps_the_current_draft_stashed(cx: &mut TestAppContext) {
        let window = conversation(cx);
        window
            .update(cx, |view, _, cx| {
                view.input
                    .update(cx, |input, cx| input.set_text("older".into(), cx));
                view.stash_or_restore(cx);
                view.input
                    .update(cx, |input, cx| input.set_text("current".into(), cx));
                view.restore_stash(0, cx);
                let stashed: Vec<_> = view
                    .follow_ups
                    .state
                    .stashes
                    .iter()
                    .map(|stash| stash.text.clone())
                    .collect();
                assert_eq!(stashed, vec!["current".to_string()]);
                let id = view.follow_ups.state.stashes[0].id.clone();
                view.remove_stash(&id, cx);
                assert!(view.follow_ups.state.stashes.is_empty());
            })
            .unwrap();
        assert_eq!(content(&window, cx), "older");
    }

    #[gpui::test]
    fn suggestions_show_only_in_an_empty_composer_and_clear_on_a_new_turn(cx: &mut TestAppContext) {
        let window = conversation(cx);
        window
            .update(cx, |view, _, cx| {
                view.show_prompt_suggestion("Run the tests".into(), cx);
                assert_eq!(
                    view.input.read(cx).ghost().map(|g| g.to_string()),
                    Some("Run the tests".into())
                );
                view.clear_prompt_suggestion(cx);
                assert!(view.input.read(cx).ghost().is_none());
                view.input
                    .update(cx, |input, cx| input.set_text("typing".into(), cx));
                view.show_prompt_suggestion("Ignored".into(), cx);
                assert!(view.input.read(cx).ghost().is_none());
            })
            .unwrap();
    }

    #[gpui::test]
    fn tab_accepts_the_suggestion_and_cmd_s_stashes(cx: &mut TestAppContext) {
        let window = conversation(cx);
        window
            .update(cx, |view, window, cx| {
                view.input.read(cx).focus_handle(cx).focus(window, cx);
                view.show_prompt_suggestion("Run cargo test".into(), cx);
            })
            .unwrap();
        cx.run_until_parked();
        cx.simulate_keystrokes(window.into(), "tab");
        assert_eq!(content(&window, cx), "Run cargo test");
        let stash = if cfg!(target_os = "macos") {
            "cmd-s"
        } else {
            "ctrl-s"
        };
        cx.simulate_keystrokes(window.into(), stash);
        assert_eq!(content(&window, cx), "");
        window
            .update(cx, |view, _, _| {
                assert_eq!(view.follow_ups.state.stashes[0].text, "Run cargo test");
            })
            .unwrap();
        cx.simulate_keystrokes(window.into(), stash);
        assert_eq!(content(&window, cx), "Run cargo test");
    }
}
