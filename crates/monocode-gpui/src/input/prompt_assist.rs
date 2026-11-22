//! Composer prompt assists: the suggested next prompt (ghost text, Tab to
//! accept), Up/Down recall of submitted prompts and the Cmd/Ctrl+S stash key.
use super::Input;
use gpui::{App, Context, SharedString, Window, actions};
use std::rc::Rc;

actions!(composer, [AcceptGhost, DismissGhost, StashDraft]);

/// Cmd/Ctrl+S in a composer that stashes drafts.
pub struct StashRequested;

/// Returns the session's submitted prompts, oldest first. Read lazily, only
/// when the user starts a recall, so long transcripts cost nothing per frame.
pub(crate) type PromptHistory = Rc<dyn Fn(&App) -> Vec<String>>;

pub(super) struct Recall {
    entries: Vec<String>,
    index: usize,
    draft: String,
}

impl Input {
    pub(crate) fn set_ghost(&mut self, text: Option<String>, cx: &mut Context<Self>) {
        let text = text
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .map(SharedString::from);
        if self.ghost != text {
            self.ghost = text;
            cx.notify();
        }
    }
    pub(crate) fn ghost(&self) -> Option<&SharedString> {
        self.ghost.as_ref().filter(|_| self.content.is_empty())
    }
    pub(crate) fn set_prompt_history(&mut self, history: Option<PromptHistory>) {
        self.history = history;
        self.recall = None;
    }
    pub(crate) fn set_stash_enabled(&mut self, enabled: bool) {
        self.stash_enabled = enabled;
    }
    /// Typing, pasting or composing drops the suggestion and leaves recall.
    pub(super) fn user_edited(&mut self) {
        self.ghost = None;
        self.recall = None;
    }
    pub(super) fn accept_ghost(&mut self, _: &AcceptGhost, _: &mut Window, cx: &mut Context<Self>) {
        if self.read_only || !self.content.is_empty() || self.is_composing() {
            cx.propagate();
            return;
        }
        let Some(text) = self.ghost.take() else {
            cx.propagate();
            return;
        };
        self.set_text(text.to_string(), cx);
    }
    pub(super) fn dismiss_ghost(
        &mut self,
        _: &DismissGhost,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.ghost().is_some() {
            self.ghost = None;
            cx.notify();
        } else {
            cx.propagate();
        }
    }
    pub(super) fn stash_draft(&mut self, _: &StashDraft, _: &mut Window, cx: &mut Context<Self>) {
        if self.stash_enabled && !self.is_composing() {
            cx.emit(StashRequested);
        } else {
            cx.propagate();
        }
    }
    /// Up on the first line (from an empty composer or the very start) walks to
    /// older prompts; Down on the last line walks back and finally restores the
    /// draft. Returns whether the key was used.
    pub(super) fn recall_step(&mut self, older: bool, cx: &mut Context<Self>) -> bool {
        if self.code.is_some() || self.single_line || self.read_only || self.is_composing() {
            return false;
        }
        let cursor = self.cursor();
        let first_line = !self.content[..cursor].contains('\n');
        let last_line = !self.content[cursor..].contains('\n');
        let mut recall = match self.recall.take() {
            Some(recall) => recall,
            None if older
                && (self.content.is_empty() || (cursor == 0 && self.selection.is_empty())) =>
            {
                let Some(history) = &self.history else {
                    return false;
                };
                let mut entries = history(cx);
                entries.retain(|entry| !entry.trim().is_empty());
                entries.dedup();
                if entries.is_empty() {
                    return false;
                }
                Recall {
                    index: entries.len(),
                    entries,
                    draft: self.content.to_string(),
                }
            }
            None => return false,
        };
        if (older && !first_line) || (!older && !last_line) {
            self.recall = Some(recall);
            return false;
        }
        if older {
            if recall.index == 0 {
                self.recall = Some(recall);
                return true;
            }
            recall.index -= 1;
            let text = recall.entries[recall.index].clone();
            self.set_text(text, cx);
            self.select_range(0..0, cx);
            self.recall = Some(recall);
        } else if recall.index + 1 < recall.entries.len() {
            recall.index += 1;
            let text = recall.entries[recall.index].clone();
            self.set_text(text, cx);
            self.recall = Some(recall);
        } else {
            self.set_text(recall.draft, cx);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext as _, TestAppContext};

    fn composer(cx: &mut TestAppContext, history: Vec<&'static str>) -> gpui::Entity<Input> {
        cx.update(|cx| {
            cx.new(|cx| {
                let mut input = Input::new(cx);
                let history: PromptHistory =
                    Rc::new(move |_| history.iter().map(|s| s.to_string()).collect());
                input.set_prompt_history(Some(history));
                input
            })
        })
    }

    #[gpui::test]
    fn up_recalls_older_prompts_and_down_restores_the_draft(cx: &mut TestAppContext) {
        let input = composer(cx, vec!["first", "second", "second", "third"]);
        input.update(cx, |input, cx| {
            assert!(input.recall_step(true, cx));
            assert_eq!(input.content.as_ref(), "third");
            assert!(input.recall_step(true, cx));
            assert_eq!(input.content.as_ref(), "second");
            assert!(input.recall_step(true, cx));
            assert_eq!(input.content.as_ref(), "first");
            // The oldest prompt holds.
            assert!(input.recall_step(true, cx));
            assert_eq!(input.content.as_ref(), "first");
            assert!(input.recall_step(false, cx));
            assert_eq!(input.content.as_ref(), "second");
            assert!(input.recall_step(false, cx));
            assert!(input.recall_step(false, cx));
            assert_eq!(input.content.as_ref(), "");
            assert!(input.recall.is_none());
            assert!(!input.recall_step(false, cx));
        });
    }

    #[gpui::test]
    fn a_draft_only_recalls_from_its_start_and_comes_back(cx: &mut TestAppContext) {
        let input = composer(cx, vec!["older"]);
        input.update(cx, |input, cx| {
            input.set_text("line one\nline two".into(), cx);
            // The caret sits at the end of a multi-line draft: Up moves the caret.
            assert!(!input.recall_step(true, cx));
            input.select_range(0..0, cx);
            assert!(input.recall_step(true, cx));
            assert_eq!(input.content.as_ref(), "older");
            assert!(input.recall_step(false, cx));
            assert_eq!(input.content.as_ref(), "line one\nline two");
        });
    }

    #[gpui::test]
    fn editing_a_recalled_prompt_leaves_recall(cx: &mut TestAppContext) {
        let input = composer(cx, vec!["older"]);
        input.update(cx, |input, cx| {
            assert!(input.recall_step(true, cx));
            input.user_edited();
            assert!(input.recall.is_none());
            assert!(!input.recall_step(false, cx));
            assert_eq!(input.content.as_ref(), "older");
        });
    }

    #[gpui::test]
    fn ghost_shows_only_while_empty_and_text_clears_it(cx: &mut TestAppContext) {
        let input = composer(cx, vec![]);
        input.update(cx, |input, cx| {
            input.set_ghost(Some("  Run the tests  ".into()), cx);
            assert_eq!(input.ghost().map(|g| g.as_ref()), Some("Run the tests"));
            assert_eq!(input.display_text().0.as_ref(), "Run the tests");
            input.set_text("typed".into(), cx);
            assert!(input.ghost().is_none());
            assert_eq!(input.display_text().0.as_ref(), "typed");
            input.set_ghost(Some("   ".into()), cx);
            assert!(input.ghost.is_none());
        });
    }
}
