use super::*;
use gpui::{
    Bounds, TestAppContext, VisualTestContext, WindowBounds, WindowHandle, WindowOptions, point,
    size,
};

fn window(section: Section, cx: &mut TestAppContext) -> WindowHandle<SettingsView> {
    let window = cx.update(|cx| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    point(px(0.), px(0.)),
                    size(px(1200.), px(800.)),
                ))),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    let mut view = SettingsView::new(
                        section,
                        Preferences::default(),
                        SettingsContext::default(),
                        cx,
                    );
                    view.discovery_task = None;
                    view.discovery_loading = false;
                    view
                })
            },
        )
        .unwrap()
    });
    cx.refresh().unwrap();
    cx.run_until_parked();
    window
}

#[gpui::test]
fn language_restart_status_tracks_effective_language(cx: &mut TestAppContext) {
    let (current, other) = if crate::i18n::needs_restart("en") {
        ("zh-CN", "en")
    } else {
        ("en", "zh-CN")
    };
    let window = window(Section::General, cx);
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    for (language, pending) in [(current, false), (other, true), (current, false)] {
        window
            .update(visual, |view, _, cx| {
                view.prefs.language = language.into();
                cx.notify();
            })
            .unwrap();
        visual.run_until_parked();
        assert_eq!(
            visual
                .debug_bounds("settings-language-restart-required")
                .is_some(),
            pending
        );
    }
}

#[gpui::test]
fn language_select_supports_keyboard_and_narrow_layout(cx: &mut TestAppContext) {
    cx.update(|cx| {
        crate::input::bind(cx);
        crate::popover::bind(cx);
    });
    let window = window(Section::General, cx);
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    let row = visual.debug_bounds("settings-row-language").unwrap();
    let select = visual.debug_bounds("settings-select-language").unwrap();
    assert!(select.top() >= row.top() && select.bottom() <= row.bottom());
    window
        .update(visual, |view, window, cx| {
            window.focus(&view.focus, cx);
            let trigger = view
                .select_triggers
                .borrow()
                .get("language")
                .unwrap()
                .clone();
            for _ in 0..20 {
                window.focus_next(cx);
                if trigger.is_focused(window) {
                    break;
                }
            }
            assert!(trigger.is_focused(window), "language is reachable by Tab");
        })
        .unwrap();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    visual.simulate_keystrokes("down enter");
    visual.run_until_parked();
    window
        .update(visual, |view, window, _| {
            assert_eq!(view.prefs.language(), "en");
            assert!(view.select_triggers.borrow()["language"].is_focused(window));
        })
        .unwrap();
    visual.simulate_resize(size(px(500.), px(800.)));
    visual.run_until_parked();
    visual.update(|window, cx| window.simulate_next_frame(cx));
    visual.run_until_parked();
    let row = visual.debug_bounds("settings-row-language").unwrap();
    let select = visual.debug_bounds("settings-select-language").unwrap();
    assert!(select.left() >= row.left() && select.right() <= row.right());
    assert!(select.bottom() <= row.bottom());
}

#[gpui::test]
fn shared_select_keeps_non_language_values_on_keyboard_selection(cx: &mut TestAppContext) {
    cx.update(|cx| {
        crate::input::bind(cx);
        crate::popover::bind(cx);
    });
    let window = window(Section::Appearance, cx);
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    window
        .update(visual, |view, window, cx| {
            window.focus(
                view.select_triggers
                    .borrow()
                    .get("workspace-theme")
                    .unwrap(),
                cx,
            );
        })
        .unwrap();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    visual.simulate_keystrokes("down enter");
    visual.run_until_parked();
    window
        .update(visual, |view, _, _| {
            assert_eq!(view.prefs.workspace_theme, "harbor")
        })
        .unwrap();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    window
        .update(visual, |view, _, _| {
            assert_eq!(view.prefs.workspace_theme, "harbor")
        })
        .unwrap();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    visual.simulate_input("moss");
    visual.run_until_parked();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    window
        .update(visual, |view, _, _| {
            assert_eq!(view.prefs.workspace_theme, "moss")
        })
        .unwrap();
}

fn assert_active_option(window: &mut Window, id: &str, label: &str, selected: bool) {
    let tree: serde_json::Value =
        serde_json::from_str(&window.debug_a11y_tree_json().expect("accessibility tree")).unwrap();
    let active = tree["active_descendant_focus"]
        .as_str()
        .expect("active option");
    let node = &tree["nodes"][active];
    assert!(node["element_id"].as_str().unwrap().contains(id), "{node}");
    assert_eq!(node["aria"]["label"], label);
    assert_eq!(node["aria"]["selected"], selected);
    let focused = tree["gpui_focus"].as_str().expect("keyboard focus");
    assert!(
        tree["nodes"][focused]["element_id"]
            .as_str()
            .unwrap()
            .contains("select-list-")
    );
    assert_ne!(active, focused);
    eprintln!(
        "a11y active={node} keyboard_focus={}",
        tree["nodes"][focused]
    );
}

#[gpui::test]
fn select_accessibility_short_list_reports_keyboard_activity(cx: &mut TestAppContext) {
    crate::i18n::configure_thread_for_test(crate::i18n::Language::SimplifiedChinese);
    cx.update(|cx| {
        crate::input::bind(cx);
        crate::popover::bind(cx);
    });
    let window = window(Section::General, cx);
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    let changes = std::rc::Rc::new(std::cell::Cell::new(0));
    let recorded = changes.clone();
    let (next, previous) = window
        .update(visual, |view, window, cx| {
            window.activate_accessibility_for_test();
            let entity = cx.entity();
            cx.subscribe(&entity, move |_, _, event: &SettingsEvent, _| {
                if matches!(event, SettingsEvent::Changed(_)) {
                    recorded.set(recorded.get() + 1);
                }
            })
            .detach();
            let trigger = view.select_triggers.borrow()["language"].clone();
            window.focus(&trigger, cx);
            window.focus_next(cx);
            let next = window.focused(cx).unwrap();
            window.focus(&trigger, cx);
            window.focus_prev(cx);
            let previous = window.focused(cx).unwrap();
            window.focus(&trigger, cx);
            (next, previous)
        })
        .unwrap();
    visual.run_until_parked();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    window
        .update(visual, |_, window, _| {
            assert_active_option(window, "select-language-0", "跟随系统", true)
        })
        .unwrap();
    visual.simulate_keystrokes("down");
    visual.run_until_parked();
    window
        .update(visual, |_, window, _| {
            assert_active_option(window, "select-language-1", "English", false)
        })
        .unwrap();
    visual.simulate_keystrokes("escape");
    visual.run_until_parked();
    window
        .update(visual, |view, window, _| {
            assert_eq!(view.prefs.language(), "system");
            assert!(view.select_triggers.borrow()["language"].is_focused(window));
        })
        .unwrap();
    visual.simulate_keystrokes("space down tab");
    visual.run_until_parked();
    window
        .update(visual, |view, window, _| {
            assert_eq!(view.prefs.language(), "system");
            assert!(view.select_open.is_none());
            assert!(next.is_focused(window), "Tab advances from the trigger");
        })
        .unwrap();
    window
        .update(visual, |view, window, cx| {
            window.focus(&view.select_triggers.borrow()["language"], cx)
        })
        .unwrap();
    visual.simulate_keystrokes("enter down shift-tab");
    visual.run_until_parked();
    window
        .update(visual, |view, window, _| {
            assert_eq!(view.prefs.language(), "system");
            assert!(previous.is_focused(window), "Shift-Tab moves backward");
        })
        .unwrap();
    assert_eq!(
        changes.get(),
        0,
        "dismissal never commits the highlighted option"
    );
    window
        .update(visual, |view, window, cx| {
            window.focus(&view.select_triggers.borrow()["language"], cx)
        })
        .unwrap();
    visual.simulate_keystrokes("space down space");
    visual.run_until_parked();
    window
        .update(visual, |view, window, _| {
            assert_eq!(view.prefs.language(), "en");
            assert!(view.select_triggers.borrow()["language"].is_focused(window));
        })
        .unwrap();
    assert_eq!(changes.get(), 1, "Space commits exactly once");
}

#[gpui::test]
fn select_accessibility_search_list_reports_keyboard_activity(cx: &mut TestAppContext) {
    crate::i18n::configure_thread_for_test(crate::i18n::Language::SimplifiedChinese);
    cx.update(|cx| {
        crate::input::bind(cx);
        crate::popover::bind(cx);
    });
    let window = window(Section::Appearance, cx);
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    let changes = std::rc::Rc::new(std::cell::Cell::new(0));
    let recorded = changes.clone();
    window
        .update(visual, |view, window, cx| {
            window.activate_accessibility_for_test();
            let entity = cx.entity();
            cx.subscribe(&entity, move |_, _, event: &SettingsEvent, _| {
                if matches!(event, SettingsEvent::Changed(_)) {
                    recorded.set(recorded.get() + 1);
                }
            })
            .detach();
            window.focus(&view.select_triggers.borrow()["workspace-theme"], cx);
        })
        .unwrap();
    visual.run_until_parked();
    visual.simulate_keystrokes("enter down");
    visual.run_until_parked();
    window
        .update(visual, |_, window, _| {
            assert_active_option(window, "select-workspace-theme-1", "Harbor", false)
        })
        .unwrap();
    visual.simulate_keystrokes("up");
    visual.run_until_parked();
    window
        .update(visual, |_, window, _| {
            assert_active_option(window, "select-workspace-theme-0", "默认", true)
        })
        .unwrap();
    visual.simulate_keystrokes("down enter");
    visual.run_until_parked();
    window
        .update(visual, |view, window, _| {
            assert_eq!(view.prefs.workspace_theme, "harbor");
            assert!(view.select_triggers.borrow()["workspace-theme"].is_focused(window));
        })
        .unwrap();
    assert_eq!(changes.get(), 1, "Enter commits exactly once");
    let next = window
        .update(visual, |view, window, cx| {
            let trigger = view.select_triggers.borrow()["workspace-theme"].clone();
            window.focus_next(cx);
            let next = window.focused(cx).unwrap();
            window.focus(&trigger, cx);
            next
        })
        .unwrap();
    visual.simulate_keystrokes("enter down tab");
    visual.run_until_parked();
    window
        .update(visual, |view, window, _| {
            assert_eq!(view.prefs.workspace_theme, "harbor");
            assert!(view.select_open.is_none());
            assert!(next.is_focused(window));
        })
        .unwrap();
    window
        .update(visual, |view, window, cx| {
            window.focus(&view.select_triggers.borrow()["workspace-theme"], cx)
        })
        .unwrap();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    visual.simulate_keystrokes("space");
    visual.run_until_parked();
    window
        .update(visual, |view, _, _| {
            assert!(
                view.select_open.is_some(),
                "Space in the filter does not select an option"
            )
        })
        .unwrap();
    visual.simulate_keystrokes("down up up");
    visual.run_until_parked();
    window
        .update(visual, |view, window, cx| {
            assert!(
                view.select_filter
                    .read(cx)
                    .focus_handle(cx)
                    .is_focused(window)
            );
        })
        .unwrap();
    visual.simulate_input("moss");
    visual.run_until_parked();
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    window
        .update(visual, |view, _, _| {
            assert_eq!(view.prefs.workspace_theme, "moss")
        })
        .unwrap();
    assert_eq!(
        changes.get(),
        2,
        "Tab did not commit the unconfirmed option"
    );
    visual.simulate_keystrokes("enter tab");
    visual.run_until_parked();
    window
        .update(visual, |view, window, _| {
            assert_eq!(view.prefs.workspace_theme, "moss");
            assert!(view.select_open.is_none());
            assert!(next.is_focused(window), "Tab also leaves the filter");
        })
        .unwrap();
    assert_eq!(changes.get(), 2);
}

#[gpui::test]
fn chinese_worktree_confirmation_previews_the_pending_action(cx: &mut TestAppContext) {
    crate::i18n::configure_thread_for_test(crate::i18n::Language::SimplifiedChinese);
    let window = window(Section::General, cx);
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    window
        .update(visual, |view, window, cx| {
            window.activate_accessibility_for_test();
            view.ctx.sessions = vec![
                (1, "/tmp/tree".into(), false),
                (2, "/tmp/tree/nested".into(), false),
            ];
            view.open_management(
                management::Target::DeleteTree(data::Tree {
                    tree: monocode_core::git::Worktree {
                        path: "/tmp/tree".into(),
                        ..Default::default()
                    },
                    locked: false,
                    dirty: Some(false),
                    unpushed: 0,
                }),
                window,
                cx,
            );
        })
        .unwrap();
    visual.run_until_parked();
    for (deleting, expected) in [
        (
            false,
            "使用此工作树的 2 个会话将保留。请选择分支或工作树以继续这些会话。",
        ),
        (true, "使用此工作树的 2 个会话将被永久删除。"),
    ] {
        window
            .update(visual, |view, window, _| {
                assert_eq!(view.management.as_ref().unwrap().delete_sessions, deleting);
                assert_eq!(
                    crate::i18n::format(
                        "settings-sessions-using-this-worktree-are",
                        &[
                            ("count", crate::system_format::unsigned(2)),
                            (
                                "value",
                                if deleting {
                                    crate::i18n::text("settings-worktree-permanently-deleted")
                                } else {
                                    crate::i18n::text("settings-worktree-kept")
                                }
                            ),
                        ]
                    ),
                    expected
                );
                let tree = window.debug_a11y_tree_json().unwrap();
                assert!(tree.contains(if deleting {
                    "删除工作树和会话"
                } else {
                    "删除工作树"
                }));
            })
            .unwrap();
        if !deleting {
            let toggle = visual
                .debug_bounds("settings-toggle-delete-worktree-sessions")
                .unwrap();
            visual.simulate_click(toggle.center(), Default::default());
            visual.run_until_parked();
        }
    }
}

#[gpui::test]
fn workspace_style_is_only_offered_under_the_new_layout(cx: &mut TestAppContext) {
    let window = window(Section::Appearance, cx);
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    for classic in [false, true, false] {
        window
            .update(visual, |view, _, cx| {
                view.ctx.classic = classic;
                view.ctx.new_layout_boxy = true;
                cx.notify();
            })
            .unwrap();
        visual.run_until_parked();
        assert_eq!(
            visual
                .debug_bounds("settings-new-layout-style-boxy")
                .is_some(),
            !classic
        );
        assert_eq!(
            visual
                .debug_bounds("settings-new-layout-style-rounded")
                .is_some(),
            !classic
        );
        assert!(
            visual
                .debug_bounds("settings-collapsed-project-rail-hidden")
                .is_none()
        );
    }
}

#[gpui::test]
fn skills_document_has_its_own_full_height_pane_and_compact_actions(cx: &mut TestAppContext) {
    let window = window(Section::Skills, cx);
    window
        .update(cx, |view, _, cx| {
            let path = std::path::PathBuf::from("/project/.agents/skills/review/SKILL.md");
            view.discovery.skills = vec![data::Skill {
                name: "review".into(),
                description: "Review changes".into(),
                path: path.clone(),
                scope: "project".into(),
                provider: "agents".into(),
            }];
            view.skill_preview = Some(path);
            view.skill_preview_text = Some(Ok(skill_document::DocumentData::new(format!(
                "---\nname: review\n---\n# Review\n\n{}",
                "Instructions.\n\n".repeat(150)
            ))));
            cx.notify();
        })
        .unwrap();
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    visual.run_until_parked();
    let left = visual.debug_bounds("skills-list-scroll").unwrap();
    let right = visual.debug_bounds("skill-preview-pane").unwrap();
    let body = visual.debug_bounds("skill-document").unwrap();
    assert_eq!(left.size.height, px(800.));
    assert_eq!(right.size.height, px(800.));
    assert!(right.left() >= left.right());
    assert!(body.bottom() <= right.bottom());
    assert!(body.size.height > px(600.));
    assert!(visual.debug_bounds("transcript").is_none());
    for id in [
        "skill-preview-action",
        "skill-copy-action",
        "skill-reveal-action",
    ] {
        assert_eq!(
            visual.debug_bounds(id).unwrap().size,
            size(px(20.), px(20.))
        );
    }
    let source = visual
        .debug_bounds("settings-skill-document-source")
        .unwrap();
    assert!(source.size.width < px(100.));
    visual.simulate_click(source.center(), Default::default());
    visual.run_until_parked();
    window
        .update(visual, |view, _, _| {
            assert_eq!(view.prefs.skill_source_paths.len(), 1)
        })
        .unwrap();
    visual.simulate_resize(size(px(700.), px(800.)));
    visual.run_until_parked();
    // Deliver the frame that applies the measured Skills container breakpoint.
    visual.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    visual.run_until_parked();
    let left = visual.debug_bounds("skills-list-scroll").unwrap();
    let right = visual.debug_bounds("skill-preview-pane").unwrap();
    assert!(
        right.top() >= left.bottom(),
        "list={left:?}, preview={right:?}"
    );
    assert_eq!(left.size.width, px(700.));
    assert_eq!(right.size.width, px(700.));
    assert!(visual.debug_bounds("skill-document").unwrap().size.height > px(200.));
}

#[gpui::test]
fn mono_mascot_is_beside_name_and_sidebar_switch_stays_visible(cx: &mut TestAppContext) {
    let window = window(Section::Monos, cx);
    window
        .update(cx, |view, _, cx| {
            view.ctx.monos = vec![MonoSetting {
                id: "layout-mono".into(),
                name: "Big Boss".into(),
                mascot: "ghost".into(),
                color: 0xee99bb,
                projects: vec!["monocode".into(), "mono-web".into()],
                show: false,
                folders: false,
            }];
            cx.notify();
        })
        .unwrap();
    let visual = &mut VisualTestContext::from_window(window.into(), cx);
    visual.run_until_parked();
    let mascot = visual.debug_bounds("settings-mono-mascot").unwrap();
    let switch = visual
        .debug_bounds("settings-toggle-mono-sidebar-show")
        .unwrap();
    let reset = visual.debug_bounds("settings-mono-reset").unwrap();
    assert_eq!(mascot.size, size(px(16.), px(16.)));
    assert_eq!(switch.size, size(px(36.), px(20.)));
    let name = visual.debug_bounds("settings-mono-name").unwrap();
    assert!(mascot.right() < name.left());
    assert!(mascot.left() < px(200.));
    assert!(switch.left() > px(800.));
    assert!(switch.right() <= reset.left());
    assert!(reset.right() <= px(1200.));
    assert!((mascot.top() - switch.top()).abs() < px(4.));
    assert!(
        visual
            .debug_bounds("settings-toggle-mono-sidebar-folders")
            .is_none()
    );
    visual.simulate_click(reset.center(), Default::default());
    visual.run_until_parked();
    assert!(visual.debug_bounds("settings-management-dialog").is_some());
    window
        .update(visual, |view, window, cx| {
            view.close_management(window, cx);
            cx.notify();
        })
        .unwrap();
    visual.simulate_resize(size(px(700.), px(800.)));
    visual.run_until_parked();
    let switch = visual
        .debug_bounds("settings-toggle-mono-sidebar-show")
        .unwrap();
    assert_eq!(switch.size, size(px(36.), px(20.)));
    assert!(switch.right() <= px(700.));
}
