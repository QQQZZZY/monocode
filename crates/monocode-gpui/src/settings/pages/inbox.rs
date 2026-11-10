use crate::{
    settings::{SettingsEvent, SettingsView, widgets::secondary_button},
    theme::*,
};
use gpui::{AnyElement, Context, SharedString, div, prelude::*, px};
impl SettingsView {
    pub(super) fn inbox_page(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut page = vec![self.project_notification_settings(cx)];
        for (id, title, description, label, hint) in [
            (
                "github",
                "GitHub",
                "settings-inbox-pull-requests-reviews-and-issues-read-through-the-github-cli",
                "settings-inbox-connection",
                "settings-inbox-install-github-cli-from-cli-github-com-run-gh-auth-login-in-a-terminal-then",
            ),
            (
                "gitlab",
                "GitLab",
                "settings-inbox-merge-requests-from-gitlab-com-or-a-self-managed-instance",
                "settings-inbox-connection",
                "settings-inbox-connect-gitlab-com-or-a-self-managed-gitlab-instance-use-a-personal-access",
            ),
            (
                "azuredevops",
                "ADO",
                "settings-inbox-pull-requests-and-boards-work-items-from-your-ado-organization",
                "settings-inbox-connection",
                "settings-inbox-connect-your-ado-organization-with-a-personal-access-token-boards-repos-rea",
            ),
            (
                "jira",
                "Jira",
                "settings-inbox-jira-cloud-issues-from-the-projects-you-pick",
                "settings-inbox-connection",
                "settings-inbox-connect-your-jira-cloud-site-with-your-email-and-an-api-token",
            ),
            (
                "linear",
                "Linear",
                "settings-inbox-issues-assigned-to-you-from-the-teams-you-pick",
                "settings-inbox-api-key",
                "settings-inbox-create-a-personal-api-key-in-linear-settings-security-access-disconnect-del",
            ),
        ] {
            let description = crate::i18n::text(description);
            let label = crate::i18n::text(label);
            let hint = crate::i18n::text(hint);
            let (connected, connection_label) = self
                .ctx
                .inbox_connections
                .get(id)
                .cloned()
                .unwrap_or_else(|| (false, crate::i18n::text("settings-checking-connection")));
            let status = div()
                .text_size(px(12.))
                .text_color(content(if connected { 0.7 } else { 0.5 }))
                .child(connection_label)
                .into_any_element();
            let controls = if id == "github" {
                vec![
                    status,
                    secondary_button(
                        "github-installation-guide",
                        crate::i18n::text("settings-installation-guide"),
                        false,
                        false,
                    )
                    .child(crate::i18n::text("settings-installation-guide"))
                    .on_click(|_, _, cx| cx.open_url("https://cli.github.com/"))
                    .into_any_element(),
                    secondary_button(
                        "github-check-again",
                        crate::i18n::text("settings-check-again"),
                        false,
                        false,
                    )
                    .child(crate::i18n::text("settings-check-again"))
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(SettingsEvent::ManageInboxProvider("github".into()))
                    }))
                    .into_any_element(),
                ]
            } else {
                vec![
                    status,
                    secondary_button(
                        SharedString::from(format!("connect-{id}")),
                        if connected {
                            crate::i18n::text("settings-manage")
                        } else {
                            crate::i18n::text("settings-connect")
                        },
                        false,
                        false,
                    )
                    .child(if connected {
                        crate::i18n::text("settings-manage")
                    } else {
                        crate::i18n::text("settings-connect")
                    })
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.emit(SettingsEvent::ManageInboxProvider(id.into()))
                    }))
                    .into_any_element(),
                ]
            };
            let row = self.row(None, false, label, Some(hint.into()), controls);
            page.push(
                self.group(
                    Some(id),
                    false,
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(icon(
                            match id {
                                "linear" => "inbox-linear",
                                "jira" => "inbox-jira",
                                "gitlab" => "inbox-gitlab",
                                "azuredevops" => "inbox-azuredevops",
                                _ => "inbox-github",
                            },
                            16.,
                        ))
                        .child(title),
                    Some(description.into()),
                    None,
                    vec![row],
                ),
            );
        }
        page
    }
}
