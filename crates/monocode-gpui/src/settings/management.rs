//! React Settings forms and their asynchronous actions.
use super::widgets::{note, secondary_button};
use super::*;
use monocode_core::provider::Kind;

#[derive(Clone)]
pub(super) enum Target {
    Cli(Kind),
    Mcp,
    Skill,
    Worktree,
    DeleteTree(data::Tree),
    ResetMono(super::MonoSetting),
}
pub(super) struct Editor {
    pub target: Target,
    pub name: Entity<Input>,
    pub body: Entity<Input>,
    pub provider: String,
    pub scope: String,
    pub base: String,
    pub existing: bool,
    pub delete_sessions: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub editing: bool,
    pub inspection: Option<monocode_core::cli::Inspection>,
    previous: Option<FocusHandle>,
    _subscriptions: Vec<Subscription>,
}
impl SettingsView {
    pub(super) fn open_management(
        &mut self,
        target: Target,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.management.as_ref().is_some_and(|e| e.busy) {
            return;
        }
        self.management_generation += 1;
        let (label, placeholder) = match target {
            Target::Cli(_) => (
                "settings-cli-path",
                crate::i18n::text("settings-auto-detected-path"),
            ),
            Target::Mcp => (
                "settings-name-optional-with-mcpservers",
                "my-server".to_owned(),
            ),
            Target::Skill => ("settings-skill-name", "skill-name".to_owned()),
            _ => ("settings-new-branch-name", "feature/my-task".to_owned()),
        };
        let name = cx.new(|cx| {
            let mut input = Input::field(crate::i18n::text(label), placeholder, content(0.35), cx);
            input.set_font_size(12.);
            input
        });
        if let Target::Cli(kind) = target {
            name.update(cx, |input, cx| {
                input.set_text(
                    self.prefs
                        .binary_paths
                        .get(kind.id())
                        .cloned()
                        .unwrap_or_default(),
                    cx,
                )
            });
        }
        if matches!(target, Target::Skill) {
            name.update(cx, |input, cx| {
                input.set_font_size(13.);
                input.set_text(monocode_core::skills::slug_name(&self.skill_query), cx);
            });
        }
        let body=cx.new(|cx|Input::multiline_field(&crate::i18n::text("settings-json-configuration"), "{\"mcpServers\":{\"my-server\":{\"command\":\"npx\",\"args\":[\"-y\",\"example-mcp\"]}}}",cx));
        let subscriptions = vec![
            cx.subscribe(&name, |_, _, _: &input::Changed, cx| cx.notify()),
            cx.subscribe(&body, |_, _, _: &input::Changed, cx| cx.notify()),
            cx.subscribe(&name, |this, _, _: &input::Submitted, cx| {
                this.submit_management(cx)
            }),
        ];
        let previous = window.focused(cx);
        if matches!(
            target,
            Target::Cli(_) | Target::DeleteTree(_) | Target::ResetMono(_)
        ) {
            window.focus(&self.focus, cx);
        } else {
            window.focus(&name.read(cx).focus_handle(cx), cx);
        }
        let provider = if self.mcp_filter == "all" {
            "claude".into()
        } else {
            self.mcp_filter.clone()
        };
        let scope = if matches!(target, Target::Skill) {
            if self.selected_project.is_empty() || self.selected_project == "~" {
                "user"
            } else {
                "project"
            }
        } else if matches!(provider.as_str(), "claude_desktop" | "codex") {
            "user"
        } else if provider == "claude" {
            "local"
        } else {
            "project"
        }
        .into();
        self.management = Some(Editor {
            target: target.clone(),
            name,
            body,
            provider,
            scope,
            base: "HEAD".into(),
            existing: false,
            delete_sessions: false,
            busy: false,
            error: None,
            editing: false,
            inspection: None,
            previous,
            _subscriptions: subscriptions,
        });
        if let Target::Cli(kind) = target {
            self.inspect_cli(kind, false, cx);
        }
        cx.notify();
    }
    pub(super) fn close_management(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.management.as_ref().is_some_and(|e| e.busy) {
            return;
        }
        self.management_generation += 1;
        if let Some(focus) = self.management.take().and_then(|e| e.previous) {
            window.focus(&focus, cx);
        }
        self.select_open = None;
        cx.notify();
    }
    fn inspect_cli(&mut self, kind: Kind, save: bool, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.management else {
            return;
        };
        let draft = editor.name.read(cx).content.trim().to_owned();
        editor.busy = true;
        editor.error = None;
        let generation = self.management_generation;
        let work = cx
            .background_executor()
            .spawn(async move { monocode_core::cli::inspect(kind, Some(&draft)) });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                if this.management_generation != generation {
                    return;
                }
                if let Some(editor) = &mut this.management {
                    editor.busy = false;
                    match result {
                        Ok(inspection) => {
                            editor.inspection = Some(inspection);
                            if save {
                                let value = editor.name.read(cx).content.trim().to_owned();
                                this.update(cx, |p| {
                                    if value.is_empty() {
                                        p.binary_paths.remove(kind.id());
                                    } else {
                                        p.binary_paths.insert(kind.id().into(), value);
                                    }
                                });
                                this.management = None;
                            }
                        }
                        Err(e) => editor.error = Some(e),
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
    pub(crate) fn management_finished(
        &mut self,
        result: Result<(), String>,
        cx: &mut Context<Self>,
    ) {
        if let Some(editor) = &mut self.management {
            editor.busy = false;
            match result {
                Ok(()) => {
                    if let Some(editor) = &self.management
                        && let Target::DeleteTree(tree) = &editor.target
                        && monocode_core::files::is_equal_or_inside(
                            &self.selected_project,
                            &tree.tree.path,
                        )
                    {
                        self.selected_project = self.discovery.worktrees.main.clone();
                    }
                    self.management = None;
                    self.refresh(cx);
                }
                Err(e) => editor.error = Some(e),
            }
        }
        cx.notify();
    }
    pub(super) fn submit_management(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.management else {
            return;
        };
        if editor.busy {
            return;
        }
        if let Target::Cli(kind) = editor.target {
            self.inspect_cli(kind, true, cx);
            return;
        }
        let name = editor.name.read(cx).content.trim().to_owned();
        let body = editor.body.read(cx).content.to_string();
        let cwd = self.selected_project.clone();
        let target = editor.target.clone();
        let provider = editor.provider.clone();
        let scope = editor.scope.clone();
        let base = editor.base.clone();
        let existing = editor.existing;
        if matches!(target, Target::Worktree | Target::Skill) && name.is_empty() {
            return;
        }
        editor.busy = true;
        editor.error = None;
        if let Target::ResetMono(mono) = target {
            cx.emit(SettingsEvent::ResetMono(mono.id));
            cx.notify();
            return;
        }
        if let Target::DeleteTree(tree) = target {
            cx.emit(SettingsEvent::RemoveWorktree {
                cwd,
                path: tree.tree.path,
                delete_sessions: editor.delete_sessions,
            });
            cx.notify();
            return;
        }
        let generation = self.management_generation;
        let work = cx.background_executor().spawn(async move {
            match target {
                Target::Mcp => monocode_core::mcp::mcp_add(cwd, provider, scope, name, body),
                Target::Skill => {
                    monocode_core::skills::create_blank(&cwd, &name, scope == "user").map(|_| ())
                }
                Target::Worktree => monocode_core::git::create_project_worktree(
                    std::path::Path::new(&cwd),
                    &name,
                    Some(&base),
                    existing,
                )
                .map(|_| ()),
                _ => Ok(()),
            }
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                if this.management_generation == generation {
                    this.management_finished(result, cx);
                }
            });
        })
        .detach();
        cx.notify();
    }
    pub(super) fn management_form(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let Some(e) = &self.management else {
            return div().into_any_element();
        };
        let target = e.target.clone();
        let name = e.name.clone();
        let body = e.body.clone();
        let busy = e.busy;
        name.update(cx, |input, _| input.set_read_only(busy));
        body.update(cx, |input, _| input.set_read_only(busy));
        let error = e.error.clone();
        let provider = e.provider.clone();
        let scope = e.scope.clone();
        let existing = e.existing;
        let base = e.base.clone();
        let deleting = e.delete_sessions;
        let editing = e.editing;
        let inspection = e.inspection.clone();
        let mut rows = Vec::new();
        match &target {
            Target::ResetMono(mono) => {
                let default = monocode_core::mono::Mono {
                    name: String::new(),
                    mascot: mono.mascot.clone(),
                    ..Default::default()
                }
                .label();
                rows.push(note(crate::i18n::format(
                    "settings-its-soul-goes-back-to-the-default",
                    &[("default", (default).to_string())],
                )));
                rows.push(
                    div()
                        .text_size(px(12.))
                        .text_color(content(0.5))
                        .child(crate::i18n::text(
                            "settings-its-conversation-projects-memory-and-habits",
                        ))
                        .into_any_element(),
                );
            }
            Target::Cli(kind) => {
                let overridden = self.prefs.binary_paths.contains_key(kind.id());
                let pending = monocode_core::cli::configured(*kind)
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned())
                    != self.prefs.binary_paths.get(kind.id()).cloned();
                rows.push(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_3()
                        .text_size(px(12.))
                        .child(crate::i18n::format(
                            "settings-cli",
                            &[("name", kind.label().to_string())],
                        ))
                        .child(
                            div().text_size(px(10.)).text_color(content(0.5)).child(
                                crate::i18n::format(
                                    "settings-global-path",
                                    &[(
                                        "value",
                                        (if error.is_some() {
                                            crate::i18n::text("settings-needs-attention")
                                        } else if pending {
                                            crate::i18n::text("settings-restart-required")
                                        } else if overridden {
                                            crate::i18n::text("settings-configured")
                                        } else {
                                            crate::i18n::text("settings-auto-detected")
                                        })
                                        .to_string(),
                                    )],
                                ),
                            ),
                        )
                        .into_any_element(),
                );
                if editing {
                    rows.push(form_field("settings-cli-path", name, 32.));
                    rows.push(note(crate::i18n::text(
                        "settings-enter-the-absolute-path-to-the-cli",
                    )));
                } else {
                    rows.push(
                        div()
                            .p_2()
                            .rounded_md()
                            .border_1()
                            .border_color(content(0.1))
                            .bg(content(0.03))
                            .text_size(px(10.))
                            .child(
                                div().font_family(crate::fonts::mono()).child(
                                    inspection
                                        .as_ref()
                                        .map(|i| i.path.display().to_string())
                                        .unwrap_or_else(|| {
                                            if busy {
                                                crate::i18n::text(
                                                    "settings-checking-the-selected-cli",
                                                )
                                            } else {
                                                crate::i18n::text(
                                                    "settings-cli-could-not-be-resolved",
                                                )
                                            }
                                        }),
                                ),
                            )
                            .child(div().mt_1().text_color(content(0.4)).child(
                                inspection.map(|i| i.version).unwrap_or_else(|| {
                                    if busy {
                                        crate::i18n::text("settings-checking-version")
                                    } else {
                                        crate::i18n::text("settings-retry-to-check-this-cli")
                                    }
                                }),
                            ))
                            .into_any_element(),
                    );
                }
            }
            Target::Mcp => {
                let available = self.scroll.bounds().size.width.as_f32();
                let column = ((if available > 0. {
                    available.min(560.)
                } else {
                    560.
                }) - 46.)
                    / 2.;
                rows.push(note(crate::i18n::text(
                    "settings-paste-a-server-configuration-and-choose-where-to",
                )));
                rows.push(
                    div()
                        .flex()
                        .gap_3()
                        .child(labeled_control(
                            "settings-provider",
                            self.select(
                                "mcp-provider",
                                crate::i18n::text("settings-provider"),
                                &provider,
                                [
                                    ("claude", "Claude Code"),
                                    ("claude_desktop", "Claude Desktop"),
                                    ("codex", "Codex"),
                                    ("cursor", "Cursor"),
                                    ("opencode", "OpenCode"),
                                ]
                                .into_iter()
                                .map(|(id, label)| (id.into(), label.into(), None))
                                .collect(),
                                Some(column),
                                cx,
                                |this, v, cx| {
                                    if let Some(e) = &mut this.management {
                                        e.provider = v;
                                        e.scope = match e.provider.as_str() {
                                            "claude" => "local",
                                            "codex" | "claude_desktop" => "user",
                                            _ => "project",
                                        }
                                        .into();
                                    }
                                    cx.notify();
                                },
                            ),
                        ))
                        .child(labeled_control(
                            "settings-scope",
                            self.select(
                                "mcp-scope",
                                crate::i18n::text("settings-scope"),
                                &scope,
                                scopes(&provider)
                                    .iter()
                                    .map(|s| {
                                        (
                                            s.to_string(),
                                            SharedString::from(match *s {
                                                "local" => crate::i18n::text("settings-local"),
                                                "project" => crate::i18n::text("settings-project"),
                                                _ => crate::i18n::text("settings-user"),
                                            }),
                                            None,
                                        )
                                    })
                                    .collect(),
                                Some(column),
                                cx,
                                |this, v, cx| {
                                    if let Some(e) = &mut this.management {
                                        e.scope = v;
                                    }
                                    cx.notify();
                                },
                            ),
                        ))
                        .into_any_element(),
                );
                rows.push(form_field(
                    "settings-name-optional-with-mcpservers",
                    name,
                    32.,
                ));
                rows.push(form_field("settings-json-configuration", body, 140.));
                rows.push(note(crate::i18n::text(
                    "settings-paste-one-entry-from-an-mcpservers-block",
                )));
            }
            Target::Skill => {
                rows.push(form_field("settings-skill-name", name, 32.));
                rows.push(self.segmented(
                    "skill-scope",
                    crate::i18n::text("settings-skill-scope"),
                    &scope,
                    &[
                        ("project", &crate::i18n::text("settings-project")),
                        ("user", &crate::i18n::text("settings-personal-2")),
                    ],
                    cx,
                    |this, v, cx| {
                        if let Some(e) = &mut this.management {
                            e.scope = v.into();
                        }
                        cx.notify();
                    },
                ));
                rows.push(note(crate::i18n::text(
                    "settings-creates-a-starter-skill-md-in-agents-skills",
                )));
            }
            Target::Worktree => {
                rows.push(note(crate::i18n::format(
                    "settings-an-independent-working-copy-of-existing-uncommitted",
                    &[("value", (self.selected_project).to_string())],
                )));
                rows.push(self.select(
                    "worktree-type",
                    crate::i18n::text("settings-branch-type"),
                    if existing { "existing" } else { "new" },
                    vec![
                        (
                            "new".into(),
                            crate::i18n::text("settings-create-a-new-branch").into(),
                            None,
                        ),
                        (
                            "existing".into(),
                            crate::i18n::text("settings-use-an-existing-local-branch").into(),
                            None,
                        ),
                    ],
                    None,
                    cx,
                    |this, v, cx| {
                        if let Some(e) = &mut this.management {
                            e.existing = v == "existing";
                            e.name.update(cx, |i, cx| i.set_text(String::new(), cx));
                        }
                        cx.notify();
                    },
                ));
                if existing {
                    let selected = name.read(cx).content.to_string();
                    rows.push(
                        self.select(
                            "worktree-branch",
                            crate::i18n::text("settings-existing-branch"),
                            &selected,
                            self.discovery
                                .worktrees
                                .branches
                                .iter()
                                .filter(|b| b.remote.is_none())
                                .map(|b| (b.name.clone(), b.name.clone().into(), None))
                                .collect(),
                            None,
                            cx,
                            |this, v, cx| {
                                if let Some(e) = &mut this.management {
                                    e.name.update(cx, |i, cx| i.set_text(v, cx));
                                }
                                cx.notify();
                            },
                        ),
                    );
                } else {
                    rows.push(form_field("settings-new-branch-name", name, 36.));
                    let mut options = vec![(
                        "HEAD".into(),
                        crate::i18n::text("settings-current-commit").into(),
                        None,
                    )];
                    options.extend(self.discovery.worktrees.branches.iter().map(|b| {
                        let label = b
                            .remote
                            .as_ref()
                            .map(|r| format!("{r}/{}", b.name))
                            .unwrap_or_else(|| b.name.clone());
                        (label.clone(), label.into(), None)
                    }));
                    rows.push(self.select(
                        "worktree-base",
                        crate::i18n::text("settings-start-from"),
                        &base,
                        options,
                        None,
                        cx,
                        |this, v, cx| {
                            if let Some(e) = &mut this.management {
                                e.base = v;
                            }
                            cx.notify();
                        },
                    ));
                }
                rows.push(note(crate::i18n::format(
                    "settings-created-in",
                    &[(
                        "value",
                        (self.discovery.worktrees.default_root.display()).to_string(),
                    )],
                )));
            }
            Target::DeleteTree(tree) => {
                let count = self
                    .ctx
                    .sessions
                    .iter()
                    .filter(|(_, p, _)| {
                        monocode_core::files::is_equal_or_inside(p, &tree.tree.path)
                    })
                    .count();
                rows.push(note(crate::i18n::text(
                    "settings-this-permanently-deletes-the-working-copy-and-everything",
                )));
                let mut consequences = div()
                    .rounded_lg()
                    .border_1()
                    .border_color(content(0.1))
                    .bg(content(0.05))
                    .p_3()
                    .text_size(px(12.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(tree.tree.path.clone());
                if count > 0 {
                    consequences = consequences.child(crate::i18n::format(
                        "settings-sessions-using-this-worktree-are",
                        &[
                            ("count", crate::system_format::unsigned(count as u64)),
                            (
                                "value",
                                (if deleting {
                                    crate::i18n::text("settings-worktree-permanently-deleted")
                                } else {
                                    crate::i18n::text("settings-worktree-kept")
                                })
                                .to_string(),
                            ),
                        ],
                    ));
                }
                if tree.dirty != Some(false) {
                    consequences = consequences.child(div().text_color(gpui::rgb(0xfbbf24)).child(
                        crate::i18n::text("settings-all-uncommitted-and-untracked-changes-here"),
                    ));
                }
                consequences = consequences.child(crate::i18n::format(
                    "settings-the-branch-and-its-commits-are-kept",
                    &[(
                        "value",
                        tree.tree
                            .branch
                            .as_deref()
                            .unwrap_or("worktree")
                            .to_string(),
                    )],
                ));
                if tree.unpushed > 0 {
                    consequences = consequences.child(crate::i18n::format(
                        "settings-unpublished-commits-stay-on-the-branch",
                        &[("value", format!("{}", tree.unpushed))],
                    ));
                }
                rows.push(consequences.into_any_element());
                if count > 0 {
                    rows.push(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .gap_3()
                            .p_3()
                            .rounded_lg()
                            .border_1()
                            .border_color(content(0.1))
                            .text_size(px(12.5))
                            .child(crate::i18n::text(
                                "settings-also-delete-associated-sessions",
                            ))
                            .child(self.toggle(
                                "delete-worktree-sessions",
                                crate::i18n::text("settings-also-delete-associated-sessions"),
                                deleting,
                                busy,
                                cx,
                                |this, on, cx| {
                                    if let Some(e) = &mut this.management {
                                        e.delete_sessions = on;
                                    }
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                    );
                }
            }
        }
        if let Some(error) = error {
            rows.push(
                div()
                    .id("management-error")
                    .role(Role::Alert)
                    .text_size(px(11.))
                    .text_color(gpui::rgb(0xf87171))
                    .child(error)
                    .into_any_element(),
            );
        }
        let mut actions = div().flex().items_center().justify_end().gap_2();
        if let Target::Cli(kind) = target {
            if !editing {
                if self.management.as_ref().is_some_and(|e| e.error.is_some()) {
                    actions = actions.child(
                        secondary_button(
                            "retry-cli",
                            crate::i18n::text("settings-retry-cli-inspection"),
                            false,
                            busy,
                        )
                        .child(crate::i18n::text("settings-retry"))
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.inspect_cli(kind, false, cx)),
                        ),
                    );
                }
                let path = self
                    .management
                    .as_ref()
                    .and_then(|e| e.inspection.as_ref())
                    .map(|i| i.path.clone());
                actions = actions
                    .child(
                        secondary_button(
                            "reveal-cli",
                            crate::i18n::text("settings-open-cli-location"),
                            false,
                            path.is_none(),
                        )
                        .child(crate::i18n::text("settings-open-location"))
                        .on_click(move |_, _, cx| {
                            if let Some(path) = &path {
                                cx.reveal_path(path);
                            }
                        }),
                    )
                    .child(
                        secondary_button(
                            "edit-cli",
                            crate::i18n::text("settings-edit-cli-path"),
                            false,
                            busy,
                        )
                        .child(crate::i18n::text("settings-edit-path"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            if let Some(e) = &mut this.management {
                                e.editing = true;
                                window.focus(&e.name.read(cx).focus_handle(cx), cx);
                            }
                            cx.notify();
                        })),
                    );
                rows.push(actions.into_any_element());
                return div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(rows)
                    .into_any_element();
            } else if self.prefs.binary_paths.contains_key(kind.id()) {
                actions = actions.child(
                    secondary_button(
                        "auto-cli",
                        crate::i18n::text("settings-use-auto-detected-path"),
                        false,
                        busy,
                    )
                    .child(crate::i18n::text("settings-use-auto-detected-path"))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(e) = &mut this.management {
                            e.name.update(cx, |i, cx| i.set_text(String::new(), cx));
                        }
                        this.inspect_cli(kind, true, cx);
                    })),
                );
            }
        }
        let label = match target {
            Target::Cli(_) => crate::i18n::text("settings-save-path"),
            Target::Mcp => {
                if busy {
                    crate::i18n::text("settings-adding")
                } else {
                    crate::i18n::text("settings-add-server")
                }
            }
            Target::Skill => {
                if busy {
                    crate::i18n::text("settings-creating")
                } else {
                    crate::i18n::text("settings-create-skill")
                }
            }
            Target::Worktree => crate::i18n::text("settings-create-worktree"),
            Target::ResetMono(_) => {
                if busy {
                    crate::i18n::text("settings-resetting")
                } else {
                    crate::i18n::text("settings-reset-mono")
                }
            }
            Target::DeleteTree(_) => {
                if deleting {
                    crate::i18n::text("settings-delete-worktree-and-sessions")
                } else {
                    crate::i18n::text("settings-delete-worktree-2")
                }
            }
        };
        actions = actions
            .child(
                crate::modal::disabled(
                    crate::modal::ghost_button(
                        "cancel-management",
                        crate::i18n::text("settings-cancel"),
                    ),
                    busy,
                )
                .on_click(cx.listener(|this, _, window, cx| this.close_management(window, cx))),
            )
            .child(
                crate::modal::disabled(
                    if matches!(target, Target::DeleteTree(_) | Target::ResetMono(_)) {
                        crate::modal::danger_button("submit-management", label)
                    } else {
                        crate::modal::primary_button("submit-management", label)
                    },
                    busy,
                )
                .on_click(cx.listener(|this, _, _, cx| this.submit_management(cx))),
            );
        rows.push(actions.into_any_element());
        div()
            .flex()
            .flex_col()
            .gap_3()
            .children(rows)
            .into_any_element()
    }
    pub(super) fn management_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let target = self.management.as_ref()?.target.clone();
        if matches!(target, Target::Skill) {
            return None;
        }
        let title = match target {
            Target::Cli(kind) => crate::i18n::format(
                "settings-cli-details",
                &[("name", (kind.label()).to_string())],
            ),
            Target::Mcp => crate::i18n::text("settings-add-mcp-server"),
            Target::Worktree => crate::i18n::text("settings-create-worktree"),
            Target::DeleteTree(_) => crate::i18n::text("settings-delete-worktree"),
            Target::ResetMono(ref mono) => crate::i18n::format(
                "settings-reset-to-its-defaults",
                &[("value", (mono.name).to_string())],
            ),
            _ => String::new(),
        };
        let form = self.management_form(cx);
        if let Target::Cli(kind) = target {
            let slot = self
                .binary_anchors
                .borrow()
                .get(&kind)
                .cloned()
                .unwrap_or_default();
            return Some(
                popover("cli-details", Anchor::Slot(slot))
                    .side(Side::Bottom)
                    .align(Align::Start)
                    .width(440.)
                    .on_dismiss({
                        let weak = cx.weak_entity();
                        move |window, cx| {
                            let _ = weak.update(cx, |this, cx| this.close_management(window, cx));
                        }
                    })
                    .child(div().p_3().child(form))
                    .into_any_element(),
            );
        }
        let overlay = div()
            .absolute()
            .inset_0()
            .occlude()
            .child(crate::modal::scrim("settings-management-scrim"))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .id("settings-management-dialog")
                    .debug_selector(|| "settings-management-dialog".into())
                    .role(Role::Dialog)
                    .aria_label(title.clone())
                    .w(px(if matches!(target, Target::Mcp) {
                        560.
                    } else {
                        448.
                    }))
                    .max_w_full()
                    .relative()
                    .rounded(px(crate::modal::RADIUS))
                    .border_1()
                    .border_color(content(crate::modal::BORDER))
                    .child(crate::modal::surface(crate::modal::RADIUS - 1.))
                    .child(
                        div()
                            .p_4()
                            .border_b_1()
                            .border_color(content(0.1))
                            .text_size(px(14.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_3()
                            .child(title)
                            .child(
                                secondary_button(
                                    "close-management",
                                    crate::i18n::text("settings-close-dialog"),
                                    false,
                                    self.management.as_ref().is_some_and(|e| e.busy),
                                )
                                .border_0()
                                .size_6()
                                .p_0()
                                .justify_center()
                                .child(icon("x", 14.))
                                .on_click(cx.listener(
                                    |this, _, window, cx| this.close_management(window, cx),
                                )),
                            ),
                    )
                    .child(div().p_4().child(form)),
            );
        Some(crate::modal::window_overlay(window, overlay).into_any_element())
    }
}
fn scopes(provider: &str) -> &'static [&'static str] {
    match provider {
        "claude" => &["local", "project", "user"],
        "codex" | "claude_desktop" => &["user"],
        _ => &["project", "user"],
    }
}
fn form_field(label: &str, input: Entity<Input>, height: f32) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .text_size(px(12.))
        .text_color(content(0.65))
        .child(crate::i18n::text(label))
        .child(
            div()
                .h(px(height))
                .px(px(10.))
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(content(0.1))
                .bg(content(0.04))
                .when(label == "settings-json-configuration", |d| {
                    d.font_family(crate::fonts::mono())
                })
                .child(input),
        )
        .into_any_element()
}

fn labeled_control(label: &str, control: AnyElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .text_size(px(12.))
        .text_color(content(0.65))
        .child(crate::i18n::text(label))
        .child(control)
        .into_any_element()
}
