//! The native menu bar, after the Tauri app's `menu.rs`. Items dispatch GPUI
//! actions, so their shortcuts follow the live keymap and items the focused
//! view cannot handle are disabled.
use crate::{
    Quit,
    file_editor::{Find, Format, ReloadFile, Replace, Save, ToggleAutosave, ToggleWordWrap},
    input,
    shell::*,
};
use gpui::{App, KeyBinding, Menu, MenuItem, actions};

actions!(application, [Hide, HideOthers, ShowAll]);

/// Opens a Help menu link in the browser.
#[derive(Clone, Debug, PartialEq, gpui::Action)]
#[action(namespace = application, no_json)]
pub(crate) struct OpenUrl(pub &'static str);

const REPO: &str = "https://github.com/hardbeat920/monocode";

fn window_menu_identity(display: String) -> String {
    if cfg!(target_os = "macos") {
        "Window".to_owned()
    } else {
        display
    }
}

pub(crate) fn bind(cx: &mut App) {
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(|action: &OpenUrl, cx| cx.open_url(action.0));
    if cfg!(target_os = "macos") {
        cx.bind_keys([
            KeyBinding::new("cmd-h", Hide, None),
            KeyBinding::new("cmd-alt-h", HideOthers, None),
            KeyBinding::new("cmd-m", MinimizeWindow, None),
        ]);
    }
}

/// Set once the first window has presented a frame.
struct MenuReady;
impl gpui::Global for MenuReady {}

/// Installs the menu bar, called after the first frame is on screen. Setting
/// the main menu makes AppKit check for Writing Tools, which loads a framework
/// on the main thread (about 70 ms), so launch must not wait for it.
pub(crate) fn ready(cx: &mut App) {
    cx.set_global(MenuReady);
    if let Some(prefs) = cx.try_global::<crate::settings::Preferences>().cloned() {
        install(&prefs, cx);
    }
}

pub(crate) fn install(prefs: &crate::settings::Preferences, cx: &mut App) {
    if !cx.has_global::<MenuReady>() {
        return;
    }
    cx.set_menus(vec![
        Menu::new(crate::i18n::text("app-menu-monocode-native")).items([
            MenuItem::action(crate::i18n::text("app-menu-about-monocode-native"), About),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-settings"), OpenSettings),
            MenuItem::action(
                crate::i18n::text("app-menu-check-for-updates"),
                CheckForUpdates,
            ),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-hide-monocode-native"), Hide),
            MenuItem::action(crate::i18n::text("app-menu-hide-others"), HideOthers),
            MenuItem::action(crate::i18n::text("app-menu-show-all"), ShowAll),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-quit-monocode-native"), Quit),
        ]),
        Menu::new(crate::i18n::text("app-menu-file")).items([
            MenuItem::action(crate::i18n::text("app-menu-open-project"), OpenProject),
            MenuItem::action(crate::i18n::text("app-menu-search"), FocusSearch),
            MenuItem::action(crate::i18n::text("app-menu-go-to-file"), GoToFile),
            MenuItem::action(
                crate::i18n::text("app-menu-command-palette"),
                OpenCommandPalette,
            ),
            MenuItem::action(crate::i18n::text("app-menu-find-in-files"), SearchFiles),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-save"), Save),
            MenuItem::action(crate::i18n::text("app-menu-reload-file"), ReloadFile),
            MenuItem::action(crate::i18n::text("app-menu-autosave"), ToggleAutosave)
                .checked(prefs.autosave),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-new-tab"), NewCodex),
            MenuItem::action(crate::i18n::text("app-menu-new-claude-tab"), NewClaude),
            MenuItem::action(crate::i18n::text("app-menu-new-terminal"), NewTerminal),
            MenuItem::action(
                crate::i18n::text("app-menu-split-terminal-right"),
                SplitTerminal,
            ),
            MenuItem::action(crate::i18n::text("app-menu-split-pane-right"), SplitRight),
            MenuItem::action(crate::i18n::text("app-menu-split-pane-down"), SplitDown),
            MenuItem::action(crate::i18n::text("app-menu-close-pane"), CloseTab),
            MenuItem::action(
                crate::i18n::text("app-menu-close-other-tabs"),
                CloseOtherTabs,
            ),
            MenuItem::action(crate::i18n::text("app-menu-close-all-tabs"), CloseAllTabs),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-previous-tab"), PreviousTab),
            MenuItem::action(crate::i18n::text("app-menu-next-tab"), NextTab),
            MenuItem::action(crate::i18n::text("app-menu-back"), VisitBack),
            MenuItem::action(crate::i18n::text("app-menu-forward"), VisitForward),
        ]),
        Menu::new(crate::i18n::text("app-menu-edit")).items([
            MenuItem::action(crate::i18n::text("app-menu-undo"), input::Undo),
            MenuItem::action(crate::i18n::text("app-menu-redo"), input::Redo),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-cut"), input::Cut),
            MenuItem::action(crate::i18n::text("app-menu-copy"), input::Copy),
            MenuItem::action(crate::i18n::text("app-menu-paste"), input::Paste),
            MenuItem::action(crate::i18n::text("app-menu-select-all"), input::SelectAll),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-find"), Find),
            MenuItem::action(crate::i18n::text("app-menu-replace"), Replace),
            MenuItem::action(crate::i18n::text("app-menu-format-document"), Format),
            MenuItem::action(crate::i18n::text("app-menu-word-wrap"), ToggleWordWrap)
                .checked(prefs.file_word_wrap),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-fold"), input::FoldCode),
            MenuItem::action(crate::i18n::text("app-menu-unfold"), input::UnfoldCode),
            MenuItem::action(crate::i18n::text("app-menu-fold-all"), input::FoldAll),
            MenuItem::action(crate::i18n::text("app-menu-unfold-all"), input::UnfoldAll),
        ]),
        Menu::new(crate::i18n::text("app-menu-view")).items([
            MenuItem::action(crate::i18n::text("app-menu-toggle-sidebar"), ToggleSidebar),
            MenuItem::action(
                crate::i18n::text("app-menu-toggle-session-sidebar"),
                ToggleRightPanel,
            ),
            MenuItem::action(crate::i18n::text("app-menu-toggle-layout"), ToggleLayout),
            MenuItem::action(
                crate::i18n::text("app-menu-toggle-workspace-style"),
                ToggleWorkspaceStyle,
            ),
            MenuItem::action(crate::i18n::text("app-menu-inbox"), OpenInbox),
            MenuItem::action(crate::i18n::text("app-menu-notes"), OpenNotes),
            MenuItem::action(
                crate::i18n::text("app-menu-toggle-terminal"),
                ToggleTerminal,
            ),
            MenuItem::action(crate::i18n::text("app-menu-switch-model"), SwitchModel),
            MenuItem::separator(),
            MenuItem::action(
                crate::i18n::text("app-menu-previous-session"),
                PreviousSession,
            ),
            MenuItem::action(crate::i18n::text("app-menu-next-session"), NextSession),
            MenuItem::action(
                crate::i18n::text("app-menu-previous-project"),
                PreviousProject,
            ),
            MenuItem::action(crate::i18n::text("app-menu-next-project"), NextProject),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-focus-left"), FocusPaneLeft),
            MenuItem::action(crate::i18n::text("app-menu-focus-right"), FocusPaneRight),
            MenuItem::action(crate::i18n::text("app-menu-focus-up"), FocusPaneUp),
            MenuItem::action(crate::i18n::text("app-menu-focus-down"), FocusPaneDown),
            MenuItem::separator(),
            MenuItem::action(crate::i18n::text("app-menu-zoom-in"), ZoomIn),
            MenuItem::action(crate::i18n::text("app-menu-zoom-out"), ZoomOut),
            MenuItem::action(crate::i18n::text("app-menu-reset-zoom"), ResetZoom),
            MenuItem::action(crate::i18n::text("app-menu-reload"), Reload),
            MenuItem::separator(),
            MenuItem::action(
                crate::i18n::text("app-menu-sidebar-appearance"),
                SidebarAppearance,
            ),
        ]),
        // AppKit adds its window list here once the menu is named "Window".
        Menu::new(window_menu_identity(crate::i18n::text("app-menu-window"))).items([
            MenuItem::action(crate::i18n::text("app-menu-minimize"), MinimizeWindow),
            MenuItem::action(crate::i18n::text("app-menu-zoom"), ZoomWindow),
        ]),
        Menu::new(crate::i18n::text("app-menu-help")).items([
            MenuItem::action(
                crate::i18n::text("app-menu-monocode-website"),
                OpenUrl("https://usemono.dev"),
            ),
            MenuItem::action(crate::i18n::text("app-menu-view-on-github"), OpenUrl(REPO)),
            MenuItem::separator(),
            MenuItem::action(
                crate::i18n::text("app-menu-report-a-bug"),
                OpenUrl(concat!(
                    "https://github.com/hardbeat920/monocode",
                    "/issues/new?template=bug_report.yml"
                )),
            ),
            MenuItem::action(
                crate::i18n::text("app-menu-request-a-feature"),
                OpenUrl(concat!(
                    "https://github.com/hardbeat920/monocode",
                    "/issues/new?template=feature_request.yml"
                )),
            ),
        ]),
    ]);
    #[cfg(target_os = "macos")]
    if let Some(main) = objc2::MainThreadMarker::new() {
        use objc2_app_kit::NSApplication;
        use objc2_foundation::NSString;
        let app = NSApplication::sharedApplication(main);
        let title = NSString::from_str(&crate::i18n::text("app-menu-window"));
        if let Some(item) = app
            .mainMenu()
            .and_then(|menu| menu.itemWithTitle(&NSString::from_str("Window")))
        {
            item.setTitle(&title);
        }
        if let Some(menu) = app.windowsMenu() {
            menu.setTitle(&title);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn window_menu_preserves_platform_identity() {
        let expected = if cfg!(target_os = "macos") {
            "Window"
        } else {
            "窗口"
        };
        assert_eq!(super::window_menu_identity("窗口".into()), expected);
    }
}
