use crate::{
    input::{self, Input},
    settings::{
        ModelChoice, SettingsView,
        widgets::{note, secondary_button},
    },
    theme::*,
};
use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};
use monocode_core::{
    provider::Kind,
    text_generator::{self, TextGeneratorConfig, TextGeneratorKind},
};

const TEXT_GEN_URL: &str = "text-gen-base-url";
const TEXT_GEN_MODEL: &str = "text-gen-model";
const TEXT_GEN_KEY: &str = "text-gen-api-key";

impl SettingsView {
    pub(super) fn providers_page(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        self.ensure_text_generator_fields(cx);
        let p = self.prefs.clone();
        let accounts = if let Some(center) = crate::accounts::get(cx) {
            center.update(cx, |center, cx| {
                center.mask_emails = p.mask_emails;
                center.show_remaining = p.show_remaining_usage;
                for kind in monocode_core::accounts::PROVIDERS {
                    center.load_provider(kind, false, cx);
                }
                center.settings_rows(cx)
            })
        } else {
            vec![note(crate::i18n::text(
                "settings-provider-accounts-load-with-the-workspace",
            ))]
        };
        let usage = vec![
            self.row(
                Some("show-remaining-usage"),
                true,
                crate::i18n::text("settings-show-remaining-usage"),
                Some(crate::i18n::text("settings-show-remaining-capacity-in-usage-meters").into()),
                vec![self.toggle(
                    "show-remaining-usage",
                    crate::i18n::text("settings-show-remaining-usage"),
                    p.show_remaining_usage,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.show_remaining_usage = on),
                )],
            ),
            self.row(
                Some("mask-emails"),
                false,
                crate::i18n::text("settings-mask-account-emails"),
                Some(crate::i18n::text("settings-hide-email-addresses-until-clicked").into()),
                vec![self.toggle(
                    "mask-emails",
                    crate::i18n::text("settings-mask-account-emails"),
                    p.mask_emails,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.mask_emails = on),
                )],
            ),
        ];
        let mut options = vec![(
            "global".into(),
            SharedString::from(crate::i18n::text("settings-global")),
            Some(icon("globe", 14.).into_any_element()),
        )];
        options.extend(self.ctx.projects.iter().map(|(path, label)| {
            (
                path.clone(),
                SharedString::from(label.clone()),
                Some(icon("folder", 14.).into_any_element()),
            )
        }));
        let scope = self.provider_scope.clone();
        let action = self.select(
            "provider-scope",
            crate::i18n::text("settings-provider-defaults-scope"),
            &scope,
            options,
            None,
            cx,
            |this, value, cx| {
                this.provider_scope = value;
                cx.notify();
            },
        );
        let binaries = self.discovery.binaries.clone();
        let mut clis = Vec::new();
        for (index, kind) in Kind::ALL.into_iter().enumerate() {
            let installed = binaries
                .iter()
                .find(|(k, _)| *k == kind)
                .and_then(|(_, p)| p.as_ref());
            let project = if scope == "global" {
                None
            } else {
                p.project_providers.get(&scope)
            };
            let selected = project
                .and_then(|p| p.provider.as_deref())
                .or(p.default_provider.as_deref())
                == Some(kind.id());
            let globally_hidden = p.hidden_providers.iter().any(|id| id == kind.id());
            let locked = scope != "global" && globally_hidden;
            let visible = !globally_hidden
                && !project.is_some_and(|p| p.hidden.iter().any(|id| id == kind.id()));
            let models = self
                .ctx
                .catalogs
                .get(kind.id())
                .cloned()
                .unwrap_or_default();
            let selected_model = project
                .and_then(|p| p.models.get(kind.id()))
                .or(p.default_models.get(kind.id()))
                .cloned()
                .or_else(|| models.first().map(|m| m.0.clone()));
            let label = div()
                .flex()
                .items_center()
                .gap_2()
                .child(provider_icon(kind.id(), 16.))
                .child(kind.label())
                .child({
                    let slot = self
                        .binary_anchors
                        .borrow_mut()
                        .entry(kind)
                        .or_default()
                        .clone();
                    secondary_button(
                        SharedString::from(format!("binary-{}", kind.id())),
                        crate::i18n::format(
                            "settings-show-cli-details",
                            &[("name", (kind.label()).to_string())],
                        ),
                        false,
                        false,
                    )
                    .relative()
                    .size_6()
                    .p_0()
                    .justify_center()
                    .child(slot.probe())
                    .child(icon("folder-open", 14.))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_management(
                            super::super::management::Target::Cli(kind),
                            window,
                            cx,
                        )
                    }))
                })
                .child(
                    div()
                        .rounded_full()
                        .bg(content(0.1))
                        .px(px(6.))
                        .py(px(2.))
                        .text_size(px(10.))
                        .text_color(content(0.6))
                        .child(if installed.is_some() {
                            crate::i18n::text("settings-installed")
                        } else {
                            crate::i18n::text("settings-not-installed")
                        }),
                );
            let mut controls = vec![
                secondary_button(
                    SharedString::from(format!("default-provider-{}", kind.id())),
                    if selected {
                        crate::i18n::text("settings-default")
                    } else {
                        crate::i18n::text("settings-use-by-default")
                    },
                    false,
                    selected || selected_model.is_none() || installed.is_none(),
                )
                .child(if selected {
                    crate::i18n::text("settings-default")
                } else {
                    crate::i18n::text("settings-use-by-default")
                })
                .when(
                    !selected && selected_model.is_some() && installed.is_some(),
                    |d| {
                        d.on_click(cx.listener(move |this, _, _, cx| {
                            let scope = this.provider_scope.clone();
                            this.update(cx, |p| {
                                if scope == "global" {
                                    p.default_provider = Some(kind.id().into());
                                } else {
                                    p.project_providers.entry(scope).or_default().provider =
                                        Some(kind.id().into());
                                }
                            })
                        }))
                    },
                )
                .into_any_element(),
            ];
            if let Some(model) = &selected_model {
                controls.insert(
                    0,
                    self.select(
                        kind.id(),
                        kind.label(),
                        model,
                        models
                            .iter()
                            .map(|(id, name)| (id.clone(), SharedString::from(name.clone()), None))
                            .collect(),
                        None,
                        cx,
                        move |this, model, cx| {
                            let scope = this.provider_scope.clone();
                            this.update(cx, |p| {
                                if scope == "global" {
                                    p.default_models.insert(kind.id().into(), model);
                                } else {
                                    p.project_providers
                                        .entry(scope)
                                        .or_default()
                                        .models
                                        .insert(kind.id().into(), model);
                                }
                            });
                        },
                    ),
                );
            }
            if installed.is_some() {
                controls.push(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(content(0.5))
                                .child(if locked {
                                    crate::i18n::text("settings-hidden-globally")
                                } else {
                                    crate::i18n::text("settings-show-in-picker")
                                }),
                        )
                        .child(self.toggle(
                            kind.id(),
                            crate::i18n::text("settings-show-provider-in-the-model-picker"),
                            visible,
                            locked,
                            cx,
                            move |this, on, cx| {
                                let scope = this.provider_scope.clone();
                                this.update(cx, |p| {
                                    let hidden = if scope == "global" {
                                        &mut p.hidden_providers
                                    } else {
                                        &mut p.project_providers.entry(scope).or_default().hidden
                                    };
                                    hidden.retain(|id| id != kind.id());
                                    if !on {
                                        hidden.push(kind.id().into());
                                    }
                                })
                            },
                        ))
                        .into_any_element(),
                );
            }
            let description = installed
                .map(|_| {
                    crate::i18n::format(
                        "settings-models-available",
                        &[
                            (
                                "value1",
                                crate::system_format::unsigned((models.len()) as u64),
                            ),
                            (
                                "value2",
                                (if models.len() == 1 {
                                    crate::i18n::text("settings-models-available-model")
                                } else {
                                    crate::i18n::text("settings-models-available-models")
                                })
                                .to_string(),
                            ),
                        ],
                    )
                })
                .unwrap_or_else(|| kind.unavailable_hint());
            clis.push(self.row(None, index == 0, label, Some(description.into()), controls));
        }
        if self.discovery_loading {
            clis.insert(
                0,
                note(crate::i18n::text("settings-checking-installed-clis")),
            );
        }
        // Any installed harness that can write one-off text, with the models
        // its sessions have listed; the harness default when none are loaded.
        let mut background = vec![(
            "auto".to_owned(),
            SharedString::from(crate::i18n::text("settings-automatic")),
            None::<AnyElement>,
        )];
        for kind in Kind::ALL {
            let installed = binaries.iter().any(|(k, p)| *k == kind && p.is_some());
            if !installed || !monocode_core::session_title::supports(kind) {
                continue;
            }
            match self.ctx.catalogs.get(kind.id()).filter(|m| !m.is_empty()) {
                Some(models) => background.extend(models.iter().map(|(id, name)| {
                    (
                        format!("{}:{id}", kind.id()),
                        SharedString::from(format!("{} · {name}", kind.label())),
                        Some(provider_icon(kind.id(), 14.).into_any_element()),
                    )
                })),
                None => background.push((
                    format!("{}:", kind.id()),
                    SharedString::from(crate::i18n::format(
                        "settings-default-model",
                        &[("name", (kind.label()).to_string())],
                    )),
                    Some(provider_icon(kind.id(), 14.).into_any_element()),
                )),
            }
        }
        let background_value = p
            .background_model
            .as_ref()
            .map(|c| format!("{}:{}", c.provider, c.model.as_deref().unwrap_or_default()))
            .unwrap_or_else(|| "auto".into());
        if !background.iter().any(|(v, _, _)| *v == background_value) {
            // A saved pick whose catalog is not loaded keeps its own label.
            let choice = p.background_model.clone().unwrap_or(ModelChoice {
                provider: String::new(),
                model: None,
            });
            background.push((
                background_value.clone(),
                SharedString::from(format!(
                    "{} · {}",
                    Kind::from_id(&choice.provider).label(),
                    choice
                        .model
                        .clone()
                        .unwrap_or_else(|| crate::i18n::text("settings-default-model-fallback"))
                )),
                Some(provider_icon(&choice.provider, 14.).into_any_element()),
            ));
        }
        let tg = &p.text_generator;
        let url_input = self.fields[TEXT_GEN_URL].clone();
        let model_input = self.fields[TEXT_GEN_MODEL].clone();
        let key_input = self.fields[TEXT_GEN_KEY].clone();
        let test_description: SharedString = match &self.text_gen_status {
            Some(Ok(reply)) => {
                let preview: String = reply.chars().take(48).collect();
                crate::i18n::format("settings-text-generator-connected", &[("reply", preview)])
                    .into()
            }
            Some(Err(err)) => err.clone().into(),
            None => crate::i18n::text("settings-text-generator-test-hint").into(),
        };
        let test_button = secondary_button(
            "test-text-generator",
            if self.text_gen_testing {
                crate::i18n::text("settings-text-generator-testing")
            } else {
                crate::i18n::text("settings-text-generator-test-action")
            },
            false,
            self.text_gen_testing,
        )
        .when(!self.text_gen_testing, |button| {
            button.on_click(cx.listener(|this, _, _, cx| this.test_text_generator(cx)))
        });
        let background = vec![
            self.row(
                Some("background-model"),
                true,
                crate::i18n::text("settings-background-model"),
                Some(
                    crate::i18n::text(
                        "settings-writes-session-titles-worktree-branch-names-commit",
                    )
                    .into(),
                ),
                vec![self.select(
                    "background-model",
                    crate::i18n::text("settings-background-model"),
                    &background_value,
                    background,
                    Some(240.),
                    cx,
                    |this, value, cx| {
                        this.update(cx, |p| {
                            p.background_model =
                                value.split_once(':').map(|(provider, model)| ModelChoice {
                                    provider: provider.into(),
                                    model: (!model.is_empty()).then(|| model.into()),
                                });
                        })
                    },
                )],
            ),
            self.row(
                Some("text-generator"),
                false,
                crate::i18n::text("settings-text-generator"),
                Some(
                    crate::i18n::text(
                        "settings-use-external-http-endpoint-for-titles-branches-commits-and-suggestions",
                    )
                    .into(),
                ),
                vec![self.toggle(
                    "text-generator",
                    crate::i18n::text("settings-text-generator"),
                    tg.enabled,
                    false,
                    cx,
                    |this, on, cx| {
                        this.text_gen_status = None;
                        this.update(cx, |p| p.text_generator.enabled = on);
                    },
                )],
            ),
            self.row(
                Some("text-generator-kind"),
                false,
                crate::i18n::text("settings-text-generator-protocol"),
                None,
                vec![self.segmented(
                    "text-generator-kind",
                    crate::i18n::text("settings-text-generator-protocol"),
                    tg.kind.id(),
                    &[
                        (
                            TextGeneratorKind::OpenaiCompatible.id(),
                            &crate::i18n::text("settings-openai-compatible"),
                        ),
                        (
                            TextGeneratorKind::Anthropic.id(),
                            &crate::i18n::text("settings-anthropic-api"),
                        ),
                    ],
                    cx,
                    |this, value, cx| {
                        this.text_gen_status = None;
                        this.update(cx, |p| {
                            p.text_generator.kind = TextGeneratorKind::from_id(value)
                        });
                    },
                )],
            ),
            self.row(
                Some("text-generator-base-url"),
                false,
                crate::i18n::text("settings-text-generator-base-url"),
                Some(crate::i18n::text("settings-text-generator-base-url-hint").into()),
                vec![text_gen_input_box(url_input)],
            ),
            self.row(
                Some("text-generator-model"),
                false,
                crate::i18n::text("settings-text-generator-model"),
                Some(crate::i18n::text("settings-text-generator-model-hint").into()),
                vec![text_gen_input_box(model_input)],
            ),
            self.row(
                Some("text-generator-api-key"),
                false,
                crate::i18n::text("settings-text-generator-api-key"),
                Some(crate::i18n::text("settings-text-generator-api-key-hint").into()),
                vec![text_gen_input_box(key_input)],
            ),
            self.row(
                Some("text-generator-test"),
                false,
                crate::i18n::text("settings-text-generator-test"),
                Some(test_description),
                vec![test_button.into_any_element()],
            ),
        ];
        let advanced = vec![
            self.row(
                Some("idle-agents"),
                true,
                crate::i18n::text("settings-keep-idle-agents-ready"),
                Some(crate::i18n::text("settings-keep-up-to-this-many-idle-agents").into()),
                vec![self.segmented(
                    "idle-agents",
                    crate::i18n::text("settings-keep-idle-agents-ready"),
                    &p.idle_agents.to_string(),
                    &[
                        ("0", &crate::i18n::text("settings-none")),
                        ("1", "1"),
                        ("2", "2"),
                        ("3", "3"),
                        ("5", "5"),
                    ],
                    cx,
                    |this, value, cx| {
                        if let Ok(value) = value.parse() {
                            this.update(cx, |p| p.idle_agents = value)
                        }
                    },
                )],
            ),
            self.row(
                Some("claude-hooks"),
                false,
                crate::i18n::text("settings-claude-code-hooks"),
                Some(
                    crate::i18n::text("settings-run-your-configured-claude-code-hooks-changes")
                        .into(),
                ),
                vec![self.toggle(
                    "claude-hooks",
                    crate::i18n::text("settings-claude-code-hooks"),
                    p.claude_hooks,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.claude_hooks = on),
                )],
            ),
            self.row(
                Some("claude-entrypoint"),
                false,
                crate::i18n::text("settings-identify-as-monocode-to-claude-experimental"),
                Some(crate::i18n::text("settings-report-monocode-as-the-client-instead-of").into()),
                vec![self.toggle(
                    "claude-entrypoint",
                    crate::i18n::text("settings-identify-as-monocode-to-claude"),
                    p.claude_entrypoint,
                    false,
                    cx,
                    |this, on, cx| this.update(cx, |p| p.claude_entrypoint = on),
                )],
            ),
        ];
        vec![
            self.group(
                Some("provider-accounts"),
                true,
                crate::i18n::text("settings-accounts"),
                Some(crate::i18n::text("settings-manage-sign-ins-and-choose-accounts-for").into()),
                Some(self.refresh_button(
                    "refresh-provider-accounts",
                    crate::i18n::text("settings-refresh"),
                    cx,
                )),
                accounts,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-usage-and-privacy"),
                None,
                None,
                usage,
            ),
            self.group(
                Some("agent-clis"),
                false,
                crate::i18n::text("settings-agent-clis"),
                Some(
                    crate::i18n::text("settings-choose-available-agents-and-default-models-for")
                        .into(),
                ),
                Some(action),
                clis,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-background-tasks"),
                Some(crate::i18n::text("settings-small-jobs-monocode-runs-with-a-model").into()),
                None,
                background,
            ),
            self.group(
                None,
                false,
                crate::i18n::text("settings-advanced"),
                None,
                None,
                advanced,
            ),
        ]
    }

    fn ensure_text_generator_fields(&mut self, cx: &mut Context<Self>) {
        let kind = self.prefs.text_generator.kind;
        let url_placeholder = match kind {
            TextGeneratorKind::OpenaiCompatible => "https://api.openai.com/v1",
            TextGeneratorKind::Anthropic => "https://api.anthropic.com",
        };
        let model_placeholder = match kind {
            TextGeneratorKind::OpenaiCompatible => "gpt-4o-mini",
            TextGeneratorKind::Anthropic => "claude-haiku-4-5",
        };
        for (key, label, placeholder, secret, initial) in [
            (
                TEXT_GEN_URL,
                crate::i18n::text("settings-text-generator-base-url"),
                url_placeholder.to_string(),
                false,
                self.prefs.text_generator.base_url.clone(),
            ),
            (
                TEXT_GEN_MODEL,
                crate::i18n::text("settings-text-generator-model"),
                model_placeholder.to_string(),
                false,
                self.prefs.text_generator.model.clone(),
            ),
            (
                TEXT_GEN_KEY,
                crate::i18n::text("settings-text-generator-api-key"),
                "sk-...".to_string(),
                true,
                self.prefs.text_generator.api_key.clone(),
            ),
        ] {
            if let Some(existing) = self.fields.get(key) {
                existing.update(cx, |input, cx| input.set_placeholder(placeholder, cx));
                continue;
            }
            let input = cx.new(|cx| {
                let mut input = Input::field(label, placeholder, content(0.35), cx);
                input.set_font_size(12.5);
                input.set_secret(secret);
                if !initial.is_empty() {
                    input.set_text(initial, cx);
                }
                input
            });
            self._subscriptions.push(cx.subscribe(
                &input,
                move |this, input, _: &input::Changed, cx| {
                    let value = input.read(cx).content.to_string();
                    let changed = match key {
                        TEXT_GEN_URL => this.prefs.text_generator.base_url != value,
                        TEXT_GEN_MODEL => this.prefs.text_generator.model != value,
                        TEXT_GEN_KEY => this.prefs.text_generator.api_key != value,
                        _ => false,
                    };
                    if changed {
                        this.text_gen_status = None;
                        this.update(cx, |p| match key {
                            TEXT_GEN_URL => p.text_generator.base_url = value.clone(),
                            TEXT_GEN_MODEL => p.text_generator.model = value.clone(),
                            TEXT_GEN_KEY => p.text_generator.api_key = value.clone(),
                            _ => {}
                        });
                    }
                },
            ));
            self.fields.insert(key, input);
        }
    }

    pub(in crate::settings) fn sync_text_generator_fields(
        &mut self,
        config: &TextGeneratorConfig,
        cx: &mut Context<Self>,
    ) {
        for (key, value) in [
            (TEXT_GEN_URL, &config.base_url),
            (TEXT_GEN_MODEL, &config.model),
            (TEXT_GEN_KEY, &config.api_key),
        ] {
            if let Some(input) = self.fields.get(key)
                && input.read(cx).content.as_ref() != value
            {
                input.update(cx, |input, cx| input.set_text(value.clone(), cx));
            }
        }
    }

    fn test_text_generator(&mut self, cx: &mut Context<Self>) {
        if self.text_gen_testing {
            return;
        }
        self.text_gen_testing = true;
        self.text_gen_status = None;
        let config = self.prefs.text_generator.clone();
        cx.notify();
        let task = cx
            .background_executor()
            .spawn(async move { text_generator::test_connection(&config) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.text_gen_testing = false;
                this.text_gen_status = Some(result);
                cx.notify();
            });
        })
        .detach();
    }
}

fn text_gen_input_box(input: gpui::Entity<Input>) -> AnyElement {
    div()
        .w(px(260.))
        .h(px(32.))
        .rounded_lg()
        .border_1()
        .border_color(content(0.15))
        .bg(content(0.03))
        .px(px(10.))
        .flex()
        .items_center()
        .child(input)
        .into_any_element()
}
