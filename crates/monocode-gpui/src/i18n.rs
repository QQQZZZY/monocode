//! Application-owned UI messages. System formatting lives outside this module.
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use std::cell::{Cell, RefCell};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Language {
    English,
    SimplifiedChinese,
}

/// Set once at startup on the UI thread; threads that never configure resolve it.
static START_LANGUAGE: OnceLock<Language> = OnceLock::new();

fn startup_language() -> Language {
    *START_LANGUAGE.get().unwrap_or(&Language::English)
}

fn preferred_language<'a>(
    preference: &str,
    locales: impl IntoIterator<Item = &'a str>,
) -> Language {
    match preference {
        "en" => return Language::English,
        "zh-CN" => return Language::SimplifiedChinese,
        _ => {}
    }
    for locale in locales {
        let normalized = locale.replace('_', "-").to_ascii_lowercase();
        let mut parts = normalized.split('-');
        match parts.next() {
            Some("en") => return Language::English,
            Some("zh") => {
                let subtags: Vec<_> = parts.collect();
                if subtags.contains(&"hant") {
                    continue;
                }
                if subtags.contains(&"hans")
                    || (subtags
                        .iter()
                        .all(|part| !matches!(*part, "tw" | "hk" | "mo"))
                        && (subtags.is_empty()
                            || subtags.contains(&"cn")
                            || subtags.contains(&"sg")))
                {
                    return Language::SimplifiedChinese;
                }
            }
            _ => {}
        }
    }
    Language::English
}

pub(crate) fn resolved_language(preference: &str) -> Language {
    let locales: Vec<_> = sys_locale::get_locales().collect();
    preferred_language(preference, locales.iter().map(String::as_str))
}

pub(crate) fn configure(preference: &str) {
    let mut language = resolved_language(preference);
    if language == Language::SimplifiedChinese
        && let Err(error) = Messages::load_locale(include_str!("../locales/zh-CN.ftl"), "zh-CN")
    {
        log::error!("{error}; using English");
        language = Language::English;
    }
    // The calling thread owns its selection; START_LANGUAGE keeps the process
    // startup language that unconfigured (background) threads inherit.
    THREAD_LANGUAGE.with(|current| current.set(Some(language)));
    let _ = START_LANGUAGE.set(language);
}

/// Select a language for this thread only.
///
/// Tests use this instead of [`configure`]: `START_LANGUAGE` is set once for the
/// process, so a test that configured Chinese would otherwise leave every
/// later test thread resolving Chinese regardless of test order.
#[cfg(test)]
pub(crate) fn configure_thread_for_test(language: Language) {
    THREAD_LANGUAGE.with(|current| current.set(Some(language)));
}

pub(crate) fn needs_restart(preference: &str) -> bool {
    let locales: Vec<_> = sys_locale::get_locales().collect();
    restart_required(
        preference,
        startup_language(),
        locales.iter().map(String::as_str),
    )
}

fn restart_required<'a>(
    preference: &str,
    current: Language,
    locales: impl IntoIterator<Item = &'a str>,
) -> bool {
    preferred_language(preference, locales) != current
}

/// Localize only stored stable titles; user titles remain untouched.
pub(crate) fn session_title(title: &str, provider: &str) -> String {
    if monocode_core::session_title::is_default_title(title, provider) {
        text("shell-new-session")
    } else if monocode_core::session_title::display_title(title, provider)
        == monocode_core::session_title::SECOND_OPINION_TITLE
    {
        // The second-opinion session stores the stable English title and is
        // localized here. A user who deliberately renames a session to that
        // exact text is treated the same way, as with default placeholders.
        text("shell-second-opinion")
    } else {
        monocode_core::session_title::display_title(title, provider).replace('\n', " ")
    }
}

struct Messages(FluentBundle<FluentResource>);

impl Messages {
    fn load(source: &str) -> Result<Self, String> {
        Self::load_locale(source, "en")
    }

    fn load_locale(source: &str, locale: &str) -> Result<Self, String> {
        let resource = FluentResource::try_new(source.to_owned())
            .map_err(|(_, errors)| format!("Invalid {locale} resource: {errors:?}"))?;
        let mut bundle = FluentBundle::new(vec![
            locale
                .parse()
                .map_err(|e| format!("Invalid locale: {e:?}"))?,
        ]);
        bundle.set_use_isolating(false);
        bundle
            .add_resource(resource)
            .map_err(|errors| format!("Duplicate {locale} message: {errors:?}"))?;
        Ok(Self(bundle))
    }

    fn resolve(&self, key: &str, args: Option<&FluentArgs<'_>>) -> Result<String, ()> {
        let pattern = self
            .0
            .get_message(key)
            .and_then(|message| message.value())
            .ok_or(())?;
        let mut errors = Vec::new();
        let text = self.0.format_pattern(pattern, args, &mut errors);
        if errors.is_empty() {
            Ok(text.into_owned())
        } else {
            Err(())
        }
    }
}

struct Catalog {
    english: Messages,
    translated: Option<Messages>,
}

impl Catalog {
    fn load(language: Language) -> Self {
        let english = Messages::load(include_str!("../locales/en.ftl"))
            .expect("valid built-in English messages");
        let translated = if language == Language::SimplifiedChinese {
            match Messages::load_locale(include_str!("../locales/zh-CN.ftl"), "zh-CN") {
                Ok(messages) => Some(messages),
                Err(error) => {
                    log::error!("{error}; using English");
                    None
                }
            }
        } else {
            None
        };
        Catalog {
            english,
            translated,
        }
    }

    fn resolve(&self, key: &str, args: Option<&FluentArgs<'_>>) -> Result<String, ()> {
        if let Some(translated) = &self.translated {
            match translated.resolve(key, args) {
                Ok(value) => return Ok(value),
                Err(()) => log::error!("Invalid translated UI message: {key}; using English"),
            }
        }
        self.english.resolve(key, args)
    }
}

thread_local! {
    // The UI thread owns its bundle; other threads keep the startup language.
    static THREAD_LANGUAGE: Cell<Option<Language>> = const { Cell::new(None) };
    // Rebuilt only when this thread selects a different language from the cached one.
    static MESSAGES: RefCell<Option<(Language, Catalog)>> = const { RefCell::new(None) };
}

fn with_catalog<R>(f: impl FnOnce(&Catalog) -> R) -> R {
    let language = THREAD_LANGUAGE.with(|thread| thread.get().unwrap_or_else(startup_language));
    MESSAGES.with(|messages| {
        {
            let mut messages = messages.borrow_mut();
            if messages
                .as_ref()
                .is_none_or(|(cached, _)| *cached != language)
            {
                *messages = Some((language, Catalog::load(language)));
            }
        }
        let messages = messages.borrow();
        f(&messages.as_ref().expect("catalog initialized above").1)
    })
}

pub(crate) fn init() {
    with_catalog(|_| ());
}

pub(crate) fn text(key: &str) -> String {
    message(key, None)
}

pub(crate) fn message(key: &str, args: Option<&FluentArgs<'_>>) -> String {
    with_catalog(|messages| match messages.resolve(key, args) {
        Ok(text) => text,
        Err(()) => {
            debug_assert!(false, "Invalid UI message: {key}");
            fallback(key)
        }
    })
}

/// Values have already been formatted for display, independently of UI language.
pub(crate) fn format(key: &str, values: &[(&str, String)]) -> String {
    let mut args = FluentArgs::new();
    for (name, value) in values {
        args.set(*name, value.as_str());
    }
    message(key, Some(&args))
}

fn fallback(key: &str) -> String {
    log::error!("Invalid UI message: {key}");
    key.to_owned()
}

pub(crate) fn duration(ms: i64) -> String {
    if ms <= 0 {
        return text("duration-now");
    }
    let minutes = ms / 60_000;
    let hours = minutes / 60;
    let (key, whole, rest) = if hours >= 24 {
        (
            if hours % 24 > 0 {
                "duration-days-hours"
            } else {
                "duration-days"
            },
            hours / 24,
            hours % 24,
        )
    } else if hours > 0 {
        (
            if minutes % 60 > 0 {
                "duration-hours-minutes"
            } else {
                "duration-hours"
            },
            hours,
            minutes % 60,
        )
    } else {
        ("duration-minutes", minutes, 0)
    };
    format(
        key,
        &[
            ("whole", crate::system_format::integer(whole)),
            ("rest", crate::system_format::integer(rest)),
        ],
    )
}

pub(crate) fn tokens(count: u64) -> String {
    let (value, key) = if count >= 1_000_000 {
        (count as f64 / 1_000_000., "tokens-millions")
    } else if count >= 1000 {
        (count as f64 / 1000., "tokens-thousands")
    } else {
        return crate::system_format::unsigned(count);
    };
    format(
        key,
        &[(
            "value",
            crate::system_format::decimal(value, 0, usize::from(value < 10.)),
        )],
    )
}

/// A duration or date without its unit. Callers put it in a catalog template
/// (`notes-updated`, `usage-…`) that supplies the unit for the language, so the
/// fragment itself must not be translated on its own.
pub(crate) fn relative(value: u64, now: u64) -> String {
    if value == 0 {
        return String::new();
    }
    let seconds = now.saturating_sub(value).saturating_add(500) / 1000;
    if seconds < 60 {
        return text("duration-now");
    }
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    if days >= 7 {
        return i64::try_from(value)
            .ok()
            .map(|at| crate::system_format::date(at, "MMMd", "%b %-d"))
            .unwrap_or_default();
    }
    let (key, whole, rest) = if days > 0 {
        ("duration-days", days, 0)
    } else if hours > 0 {
        (
            if minutes.is_multiple_of(60) {
                "duration-hours"
            } else {
                "duration-hours-minutes"
            },
            hours,
            minutes % 60,
        )
    } else {
        ("duration-minutes", minutes, 0)
    };
    format(
        key,
        &[
            ("whole", crate::system_format::unsigned(whole)),
            ("rest", crate::system_format::unsigned(rest)),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_counts_and_relative_dates_keep_existing_boundaries() {
        for (value, expected) in [
            (999, crate::system_format::unsigned(999)),
            (
                1000,
                format!("{}K", crate::system_format::decimal(1., 0, 1)),
            ),
            (
                1500,
                format!("{}K", crate::system_format::decimal(1.5, 0, 1)),
            ),
            (
                10_000,
                format!("{}K", crate::system_format::decimal(10., 0, 0)),
            ),
            (
                1_000_000,
                format!("{}M", crate::system_format::decimal(1., 0, 1)),
            ),
        ] {
            assert_eq!(tokens(value), expected);
        }
        let at = 1_700_000_000_000;
        assert_eq!(relative(0, at), "");
        assert_eq!(relative(at, at - 1000), "now");
        assert_eq!(relative(at, at + 59_499), "now");
        assert_eq!(
            relative(at, at + 59_500),
            format!("{}m", crate::system_format::unsigned(1))
        );
        assert_eq!(
            relative(at, at + 10_740_000),
            format!(
                "{}h {}m",
                crate::system_format::unsigned(2),
                crate::system_format::unsigned(59)
            )
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn relative_absolute_dates_follow_local_day() {
        for at in [1767310200000_u64, 1767313800000] {
            assert_eq!(
                relative(at, at + 604_800_000),
                crate::system_format::local_date_reference(at as i64, "MMMd")
            );
        }
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn relative_absolute_dates_use_local_day_in_legacy_fallback() {
        for at in [1767310200000_u64, 1767313800000] {
            let expected = chrono::DateTime::from_timestamp_millis(at as i64)
                .unwrap()
                .with_timezone(&chrono::Local)
                .format("%b %-d")
                .to_string();
            assert_eq!(relative(at, at + 604_800_000), expected);
        }
    }

    #[test]
    fn intentional_sentence_spacing_and_notice_plurals_survive_fluent() {
        assert_eq!(text("settings-tomorrow"), "Tomorrow, ");
        assert_eq!(
            text("settings-open-monocode-on-your-phone-tap"),
            "Open MonoCode on your phone, tap "
        );
        for kind in ["commit", "review", "comment"] {
            for count in [0, 1, 2] {
                let mut args = FluentArgs::new();
                args.set("count", count);
                args.set("countText", count.to_string());
                assert_eq!(
                    message(&format!("notifications-new-{kind}s"), Some(&args)),
                    format!("{count} new {kind}{}", if count == 1 { "" } else { "s" })
                );
            }
        }
    }

    #[test]
    fn english_catalog_loads() {
        init();
    }

    #[test]
    fn reconfigure_applies_to_the_calling_thread_and_rebuilds_the_catalog() {
        configure("en");
        assert_eq!(text("settings-sounds"), "Sounds");
        configure("zh-CN");
        assert_eq!(text("settings-sounds"), "声音");
        configure("en");
        assert_eq!(text("settings-sounds"), "Sounds");
    }

    #[test]
    fn threads_without_an_override_resolve_the_startup_language() {
        // A thread that never configures must keep the process startup language,
        // which is what production's background threads rely on.
        let startup = startup_language();
        let other = std::thread::spawn(|| text("settings-sounds"))
            .join()
            .unwrap();
        assert_eq!(
            other,
            match startup {
                Language::English => "Sounds",
                Language::SimplifiedChinese => "声音",
            }
        );
    }

    #[test]
    fn language_matching_respects_order_and_script() {
        use Language::*;
        for (preference, locales, expected) in [
            (
                "system",
                vec!["fr-FR", "zh-Hans-CN", "en-US"],
                SimplifiedChinese,
            ),
            ("system", vec!["zh-Hant-TW", "en-US", "zh-CN"], English),
            (
                "system",
                vec!["zh-HK", "zh-Hant-CN", "zh-SG"],
                SimplifiedChinese,
            ),
            ("system", vec!["de-DE"], English),
            ("system", vec!["zh-Hant"], English),
            ("system", vec!["zh-TW"], English),
            ("unknown", vec!["zh-CN"], SimplifiedChinese),
            ("en", vec!["zh-CN"], English),
            ("zh-CN", vec!["en-US"], SimplifiedChinese),
        ] {
            assert_eq!(preferred_language(preference, locales), expected);
        }
    }

    #[test]
    fn restart_hint_tracks_actual_language_not_preference_identity() {
        use Language::*;
        for (current, preference, locales, expected) in [
            (English, "system", vec!["en-US"], false),
            (English, "en", vec!["zh-CN"], false),
            (English, "zh-CN", vec!["en-US"], true),
            (SimplifiedChinese, "system", vec!["zh-Hans"], false),
            (SimplifiedChinese, "en", vec!["zh-CN"], true),
            (SimplifiedChinese, "zh-CN", vec!["en-US"], false),
            (SimplifiedChinese, "system", vec!["fr", "zh-Hant"], true),
        ] {
            assert_eq!(restart_required(preference, current, locales), expected);
        }
    }

    #[test]
    fn translated_failures_use_english_without_merging_locales() {
        let english =
            Messages::load("label = English\nonly-english = fallback\nresult = { $count ->\n [one] one\n *[other] other\n}")
                .unwrap();
        let translated =
            Messages::load_locale("label = 中文\nresult = { $missing }", "zh-CN").unwrap();
        let catalog = Catalog {
            english,
            translated: Some(translated),
        };
        assert_eq!(catalog.resolve("label", None).unwrap(), "中文");
        assert_eq!(catalog.resolve("only-english", None).unwrap(), "fallback");
        let mut args = FluentArgs::new();
        args.set("count", 1);
        assert_eq!(catalog.resolve("result", Some(&args)).unwrap(), "one");
    }

    #[test]
    fn copy_review_chinese_messages_keep_action_and_status_meaning() {
        let zh = Messages::load_locale(include_str!("../locales/zh-CN.ftl"), "zh-CN").unwrap();
        assert_eq!(zh.resolve("monos-on", None).unwrap(), "开启");
        assert_eq!(zh.resolve("inbox-dismissed", None).unwrap(), "已驳回");
        assert_eq!(
            zh.resolve("inbox-event-dismissed", None).unwrap(),
            "的审阅已被驳回"
        );
        let mut args = FluentArgs::new();
        args.set("value", "暂存");
        args.set("path", "src/main.rs");
        assert_eq!(
            zh.resolve("changes-changes-in", Some(&args)).unwrap(),
            "暂存 src/main.rs 中的更改"
        );
    }

    #[test]
    fn copy_review_fluent_combinations_preserve_both_languages() {
        let en = Messages::load(include_str!("../locales/en.ftl")).unwrap();
        let zh = Messages::load_locale(include_str!("../locales/zh-CN.ftl"), "zh-CN").unwrap();
        fn resolve(bundle: &Messages, key: &str, values: &[(&str, &str)]) -> String {
            let mut args = FluentArgs::new();
            for (name, value) in values {
                args.set(*name, *value);
            }
            bundle.resolve(key, Some(&args)).unwrap()
        }
        for (bundle, key, args, expected) in [
            (
                &en,
                "inbox-review-on",
                vec![("provider", "GitHub")],
                "Review on GitHub",
            ),
            (
                &zh,
                "inbox-review-on",
                vec![("provider", "GitHub")],
                "在 GitHub 上审阅",
            ),
            (
                &en,
                "inbox-open-in",
                vec![("provider", "Linear")],
                "Open in Linear",
            ),
            (
                &zh,
                "inbox-open-in",
                vec![("provider", "Linear")],
                "在 Linear 中打开",
            ),
            (
                &en,
                "shell-open",
                vec![("noun", "issue"), ("value", "123")],
                "Open issue #123",
            ),
            (
                &zh,
                "shell-open",
                vec![("noun", "议题"), ("value", "123")],
                "打开 议题 #123",
            ),
            (
                &en,
                "inbox-thread-comments",
                vec![("count", "1"), ("value1", "comment"), ("value2", "")],
                "1 comment",
            ),
            (
                &en,
                "inbox-thread-comments",
                vec![("count", "2"), ("value1", "comments"), ("value2", "")],
                "2 comments",
            ),
            (
                &zh,
                "inbox-thread-comments",
                vec![("count", "2"), ("value1", "评论"), ("value2", "")],
                "2 条评论",
            ),
            (
                &zh,
                "transcript-thinking-2",
                vec![("value", "显示"), ("summary", "分析")],
                "显示思考过程：分析",
            ),
        ] {
            assert_eq!(resolve(bundle, key, &args), expected);
        }
        for (key, english, chinese) in [
            ("shell-action-save-folders", "save folders", "保存文件夹"),
            (
                "shell-action-pin-session",
                "pin the session",
                "更新会话固定状态",
            ),
            (
                "shell-action-rename-session",
                "rename the session",
                "重命名会话",
            ),
            ("shell-action-set-reminder", "set the reminder", "设置提醒"),
            (
                "shell-action-link-session",
                "link the session",
                "更新会话关联",
            ),
            (
                "shell-action-save-session-folder",
                "save the session folder",
                "保存会话文件夹",
            ),
        ] {
            for (bundle, action, expected) in [
                (&en, english, format!("Could not {english}: error")),
                (&zh, chinese, format!("无法{chinese}：error")),
            ] {
                assert_eq!(resolve(bundle, key, &[]), action);
                assert_eq!(
                    resolve(
                        bundle,
                        "shell-could-not",
                        &[("what", action), ("error", "error")]
                    ),
                    expected
                );
            }
        }
        // The schedule bar reads "Repeats Weekly Monday 09:00": the pickers
        // carry the whole meaning, with no on/at connectors to translate.
        for (bundle, expected) in [
            (&en, "Repeats Weekly Monday 09:00"),
            (&zh, "重复 每周 星期一 09:00"),
        ] {
            let day = if std::ptr::eq(bundle, &en) {
                "Monday"
            } else {
                "星期一"
            };
            let joined = [
                resolve(bundle, "monos-repeats", &[]),
                resolve(bundle, "automations-weekly", &[]),
                day.into(),
                "09:00".into(),
            ]
            .join(" ");
            assert_eq!(joined, expected);
        }
        for (bundle, day, expected) in [
            (&en, "Tomorrow, ", "1 hour (Tomorrow, 14:00)"),
            (&zh, "明天 ", "1 小时（明天 14:00）"),
        ] {
            assert_eq!(resolve(bundle, "settings-tomorrow", &[]), day);
            assert_eq!(
                resolve(
                    bundle,
                    "settings-mute-for-hour",
                    &[("hours", "1"), ("day", day), ("time", "14:00")]
                ),
                expected
            );
        }
    }

    #[test]
    fn chinese_plural_uses_chinese_rules() {
        let messages =
            Messages::load_locale(include_str!("../locales/zh-CN.ftl"), "zh-CN").unwrap();
        for count in [0, 1, 2] {
            let mut args = FluentArgs::new();
            args.set("count", count);
            args.set("countText", count.to_string());
            assert_eq!(
                messages.resolve("keybindings-count", Some(&args)).unwrap(),
                format!("{count} 个快捷键")
            );
            assert_eq!(
                messages
                    .resolve("notifications-new-reviews", Some(&args))
                    .unwrap(),
                format!("{count} 条新审阅")
            );
        }
        assert!(
            messages
                .resolve("settings-tomorrow", None)
                .unwrap()
                .ends_with(' ')
        );
        assert!(
            messages
                .resolve("settings-open-monocode-on-your-phone-tap", None)
                .unwrap()
                .ends_with(' ')
        );
        let mut args = FluentArgs::new();
        args.set("value1", "• zsh (cargo)");
        assert_eq!(
            messages
                .resolve("shell-running-terminals-confirm", Some(&args))
                .unwrap(),
            "以下终端仍在运行：\n• zsh (cargo)\n\n仍要关闭吗？"
        );
    }

    #[test]
    fn translated_catalog_matches_english_call_contracts() {
        fn entries(source: &str) -> std::collections::BTreeMap<&str, String> {
            let mut result = std::collections::BTreeMap::new();
            let mut key = None;
            for line in source.lines() {
                if let Some((name, value)) = line.split_once(" = ")
                    && !name.starts_with(char::is_whitespace)
                    && !name.starts_with('#')
                {
                    assert!(result.insert(name, value.to_owned()).is_none(), "{name}");
                    key = Some(name);
                } else if line.starts_with(char::is_whitespace)
                    && let Some(value) = key.and_then(|key| result.get_mut(key))
                {
                    value.push_str(line);
                }
            }
            result
        }
        fn variables(value: &str) -> std::collections::BTreeSet<&str> {
            value
                .split('$')
                .skip(1)
                .map(|part| {
                    part.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .next()
                        .unwrap()
                })
                .collect()
        }
        let english = entries(include_str!("../locales/en.ftl"));
        let chinese_source = include_str!("../locales/zh-CN.ftl");
        let chinese = entries(chinese_source);
        let bundle = Messages::load_locale(chinese_source, "zh-CN").unwrap();
        assert_eq!(
            english.keys().collect::<Vec<_>>(),
            chinese.keys().collect::<Vec<_>>()
        );
        for (key, value) in &chinese {
            let original = english
                .get(key)
                .unwrap_or_else(|| panic!("extra Chinese key: {key}"));
            let allowed = variables(original);
            assert!(
                variables(value).is_subset(&allowed),
                "unexpected parameter: {key}"
            );
            let mut args = FluentArgs::new();
            for name in allowed {
                if name == "count" {
                    args.set(name, 1);
                } else {
                    args.set(name, "sample");
                }
            }
            assert!(bundle.resolve(key, Some(&args)).is_ok(), "{key}");
        }
    }

    #[test]
    fn every_catalog_message_resolves_with_its_call_contract() {
        let source = include_str!("../locales/en.ftl");
        let messages = Messages::load(source).unwrap();
        let mut entries = Vec::<(&str, String)>::new();
        for line in source.lines() {
            if !line.starts_with(char::is_whitespace) {
                if let Some((key, value)) = line.split_once(" = ") {
                    entries.push((key, value.to_owned()));
                }
            } else if let Some((_, value)) = entries.last_mut() {
                value.push_str(line);
            }
        }
        for (key, value) in entries {
            let mut args = FluentArgs::new();
            for part in value.split('$').skip(1) {
                let name = part
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .next()
                    .unwrap();
                args.set(name, "sample");
            }
            assert!(messages.resolve(key, Some(&args)).is_ok(), "{key}");
        }
    }

    #[test]
    fn binding_count_uses_english_plural_rules() {
        for (count, expected) in [(0, "0 bindings"), (1, "1 binding"), (2, "2 bindings")] {
            let mut args = FluentArgs::new();
            args.set("count", count);
            args.set("countText", count.to_string());
            assert_eq!(message("keybindings-count", Some(&args)), expected);
        }
    }

    #[test]
    fn relative_time_units_keep_english_singular_and_plural() {
        for unit in ["second", "minute", "hour", "day", "week", "month", "year"] {
            assert_eq!(text(&format!("inbox-time-unit-{unit}")), unit);
            for count in [0, 1, 2] {
                let mut args = FluentArgs::new();
                args.set("count", count);
                args.set("countText", count.to_string());
                assert_eq!(
                    message(&format!("inbox-time-{unit}"), Some(&args)),
                    format!("{count} {unit}{}", if count == 1 { "" } else { "s" })
                );
            }
        }
    }

    #[test]
    fn duration_keeps_existing_truncation() {
        for (ms, expected) in [
            (-1, "now"),
            (0, "now"),
            (1, "0m"),
            (59_999, "0m"),
            (60_000, "1m"),
            (3_600_000, "1h"),
            (3_660_000, "1h 1m"),
            (86_400_000, "1d"),
            (90_000_000, "1d 1h"),
        ] {
            assert_eq!(duration(ms), expected);
        }
    }

    #[test]
    fn dialog_line_breaks_and_detail_case_are_preserved() {
        assert_eq!(
            format(
                "shell-running-terminals-confirm",
                &[("value1", "• zsh (cargo)".into())]
            ),
            "These terminals are still running:\n• zsh (cargo)\n\nClose them anyway?"
        );
        assert_eq!(text("usage-footer-not-connected-detail"), "not connected");
        assert_eq!(
            text("usage-footer-usage-unavailable-detail"),
            "usage unavailable"
        );
        assert_eq!(text("settings-local-project-prefix"), "Local project · ");
        assert!(text("explorer-name-exists-suffix").starts_with(' '));
    }

    #[test]
    fn duplicate_messages_are_rejected() {
        assert!(Messages::load("label = First\nlabel = Second").is_err());
    }

    #[test]
    fn arguments_follow_the_call_contract() {
        let messages = Messages::load("result = { $count ->\n [one] One result\n *[other] { $count } results\n}\nlabel = Results").unwrap();
        assert!(messages.resolve("missing", None).is_err());
        assert!(messages.resolve("result", None).is_err());
        for (count, expected) in [(0, "0 results"), (1, "One result"), (2, "2 results")] {
            let mut args = FluentArgs::new();
            args.set("count", count);
            args.set("unused", "not part of the translation");
            assert_eq!(messages.resolve("result", Some(&args)).unwrap(), expected);
            assert_eq!(messages.resolve("label", Some(&args)).unwrap(), "Results");
        }
        assert_eq!(fallback("missing"), "missing");
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "Invalid UI message")]
    fn missing_message_fails_in_development() {
        text("missing-test-message");
    }
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "Invalid UI message")]
    fn missing_argument_fails_in_development() {
        text("notice-agent-finished");
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn production_resolution_errors_fall_back_to_the_key() {
        assert_eq!(text("missing-test-message"), "missing-test-message");
        assert_eq!(text("notice-agent-finished"), "notice-agent-finished");
    }
}
