# Native UI internationalization

Implementation specification, updated November 7, 2026. The language-setting
and catalog policy below supersedes the earlier English-only implementation.

## Scope

The native Rust/GPUI application embeds `locales/en.ftl` and
`locales/zh-CN.ftl`, using separate Fluent bundles with `en` and `zh-CN`
locales. Preserve existing English wording and stable semantic message keys.
All application-owned labels, settings search labels and
keywords, menus, conversation controls, auxiliary windows, notifications and
accessibility names belong at the UI message boundary. Keep action/setting IDs,
protocol values, user content, model output, logs and diagnostic details intact.
Core must not depend on GPUI or Fluent.

Default conversation titles are stable storage values, including the existing
provider-only aliases. Match and update them as data when changing provider or
processing a first prompt; never persist a translated placeholder. Localize the
default at the visible title, search and accessibility boundaries. The same rule
covers other app-generated stored values: a second-opinion session stores
`Second opinion` and a habit run stores its stable error text, with the native UI
mapping both to the current language only when displaying them. Renaming a
default starts with an empty field and a localized placeholder, so accepting an
unchanged field does not store the display text. A pending settings save must not
overwrite a title generated or renamed afterward. The placeholder update for a
remote session applies only before it is bound to its host.

The shared settings selector keeps its selected value separate from its active
option. Arrow navigation exposes the active option in the GPUI accessibility
tree; in a searchable list it moves keyboard focus from the filter to the list,
and Up from its first item returns to the filter. Enter or Space on the list
commits once, Escape cancels, and Tab/Shift-Tab leave without committing the
unconfirmed option. Space in the filter remains text input.

General → Application → Language offers System, English and 简体中文. System
is the default. The app-wide preference is stored in the shared workspace record
for the selected database, not per project. At startup, System matches the
system's preferred languages in order against English and Simplified Chinese;
explicit English or zh-CN selects that language. Unsupported languages fall back
to English, and unknown saved values safely fall back to System. Explicit
Traditional Chinese does not match Simplified Chinese. Changes are saved for the
next launch: show `settings-restart-required` (`Restart required` / `需要重启`)
only when the resolved language differs from the language selected at startup,
and never restart automatically.

Initialize the selected language before creating UI consumers, including menus,
search, auxiliary windows and notifications. Return owned text; never leak
strings or introduce a global Mutex. A missing or unresolvable Chinese message is
logged and retried in the separate English bundle; invalid Chinese resources also
fall back to English. If English cannot resolve a message, debug/tests assert and
production logs the key and returns it. Do not merge English messages into the
Chinese bundle, whose plural rules must remain Chinese. Message arguments are a
caller contract: translations may omit supplied variables, but may not require
variables absent from the English contract. Validate syntax, duplicate IDs and
actual resolution with Fluent.

macOS menu creation keeps the vendor's `Window` identity for AppKit registration
and applies the localized display title afterward. The macOS bundle declares
`en` and `zh-Hans`; the embedded Simplified Chinese catalog is `zh-CN.ftl`.

## System formatting

UI language does not select a region. Never infer `en-US` from English or derive
format settings from preferred languages. Instant timestamps, including notes,
archive and relative dates, display in the system local time zone. A date-only
value retains its business calendar day. Foundation selects date order, weekday
and month names, separators and the 12/24-hour clock from system preferences;
reminders request weekday, month, day and time without forcing a year or clock.
USD remains USD, with the existing business precision and rounding. Amounts,
exchange rates, stored timestamps, protocol serialization and protocol parsing do
not change. The earlier notes/archive UTC display policy is superseded.

The system-formatting implementation scope is macOS: reuse the Foundation bridge and current
system locale/date/time preferences. Windows and Linux system formatting remain
explicit implementation and verification gaps; existing hardcoded formatting on
those platforms is not described as system-default support. Do not add ICU4X,
a generic formatter trait, a cache framework or cross-platform infrastructure.
Any localized diff number must be measured using GPUI text shaping, including
its signs and actual font, rather than UTF-8 byte counts.

## Verification and delivery

Check both resource loads, duplicate/missing/extra keys and argument contracts,
permitted unused arguments, English fallback, and locale-specific selector
branches. Search matching and displayed labels
must share resources while retaining stable setting IDs. Check date boundaries,
invalid timestamps, large/negative integers, currency and precision preservation.
Run native formatting, Clippy, unit/integration tests and a native build. Record
actual results separately from this specification. Do not restart or overwrite
the user's app, interrupt agents, submit commits or push. A framework or a single
migrated page does not constitute completed coverage.

## Implementation boundary

The November 6 baseline embedded English only and had no language preference;
that policy is superseded by the current Scope above. `src/i18n.rs` now owns
thread-local bundles initialized before native windows and menus; `text`,
`format` and `message` use the selected-language resolver and English fallback.
Core and vendor code have no Fluent dependency or translation changes.

The migration reaches settings and their search index, keybindings and command
search, application/context menus, sidebar/project/worktree controls, file and
repository search, editor and diff controls, conversation/composer/transcript
controls, response menus, notes, Inbox, automations, Monos, Quick Composer,
notification and update surfaces, resource-pressure windows, and application-owned
accessibility labels, task progress, queued attachment captions and scheduled-habit summaries. Setting/action IDs, search scopes, persisted keybindings,
selectors and animation state identities remain separate from display messages.
Brand/model names, file paths, code, user/model content, diagnostic details and
protocol values stay raw. Shared core/runtime status values are mapped only at
native presentation boundaries. Generated model prompts keep their original text.

`src/runtime_text.rs` is that boundary for conversation status and saved error
rows. A conversation's status is a typed `RunStatus`; its table pairs each
variant with the stable English text workers and storage already use and the
catalog key that renders it, so `RunStatus::parse` (at the worker-to-view
boundary) and `runtime_text::display` share one definition. Known variants never
carry localized text in the view; `RunStatus::Raw` holds unknown runtime input
and UI-thread notices that were localized where they were produced, and display
resolves those through the provider/remote pipelines. The runtime keeps writing
stable English text to transcript `error` rows and to storage; the UI maps known
values through `runtime_text::status` when it renders them, so switching language
re-renders existing history rather than leaving a stale language in place.
Provider text, storage details and protocol-consistency diagnostics (for example
a receipt for another session) are not UX copy and pass through unchanged.
Attachment validation (`runtime_text::attachment_error`) and remote-transfer
errors resolve on the UI thread or at display, keeping the path or OS detail
raw. The editor's `input::Rejected` notices follow the same rule through
`runtime_text::editor_rejection`.

Failed tests must not change the process startup language. `i18n::configure`
sets `START_LANGUAGE` once for the whole process, so a test that calls it leaves
every later test thread resolving that language regardless of test order. Tests
select a language on their own thread with `configure_thread_for_test`; only the
test that deliberately covers startup inheritance calls `configure`.

`src/system_format.rs` uses Foundation `currentLocale`, `NSDateFormatter` and
`NSNumberFormatter`. Its callers cover timestamps and calendars, schedule/time
picker captions, relative dates, duration components, token/count and compact
chart numbers, percentages, diff statistics, media sizes and USD displays.
Formatting does not alter stored schedule values, timestamps, currencies or
amounts. Existing precision, truncation and rounding rules remain explicit at
call sites. Weekday names and automation template schedule captions follow the
system region; an hourly minute offset is labelled as a minute rather than
pretending to be a complete time of day. Unsigned integers never pass through
`f64`. Editable schedule and mute times display and accept the system clock format;
the established `HH:mm` input is also accepted, and schedules still store `HH:mm`.
Line/column identifiers and technical time-zone offsets retain their machine syntax.
The mute calendar derives month boundaries, month navigation and day numbers from
the current macOS system calendar, while preserving the selected local business
date and the existing weekday ordering. Other calendar layouts are unchanged.
System-formatted time input is checked for full consumption and the hour range of
the actual formatter; localized digits and legacy `HH:mm` remain accepted.

Changed diff widths, counted project tabs and chart numeric fitting use GPUI
text shaping of the actual display string. The submit-button animation stores message keys and
measures the resolved caption. Byte lengths still used in `TextRun.len` describe
UTF-8 ranges, not estimated visual widths.

`scripts/check-native-i18n.py`, included in `native:check`, checks each supported
catalog for duplicate IDs, key parity with English and translation variables
outside the English contract. It rejects missing literal-call keys and missing
named arguments in each locale; unused caller arguments are allowed. The
supported locale list is checked against embedded resources and macOS bundle
declarations by the Python tests. It also rejects direct translation calls inside IDs/selectors.
Fluent identifier case is recognized. Literal `message` keys are checked; `None`
arguments can be checked, while `Some` arguments are reported as unchecked.
Only direct `format` argument tuples in supported literal slices are collected.
Nested message arguments never satisfy an outer contract. The actual notification
`CATEGORIES` table is checked for existing, argument-free keys. Simple literal
branches are checked; mixed or unsupported branches retain literal-key checks and
are explicitly reported as unchecked. Dynamic arguments and macros are unchecked; this is
a limited static contract check, not a Rust parser or proof of UI coverage. Rust
tests load every resource, exercise selector branches and failures, and validate
dynamic settings/search paths. A temporary Rust AST inventory was used to audit
production expressions without mistaking early test-module declarations for the
end of a source file.

Platform and acceptance limits:

- System language lookup uses `sys-locale`: CoreFoundation preferred languages on
  macOS, `GetUserPreferredUILanguages` on Windows and locale environment
  variables on Unix. Windows/Linux language selection has not been tested on
  those platforms; Unix environment variables are not proof of every desktop's
  native language settings.
- Windows/Linux system format integration is **not implemented or verified**.
  Their legacy fallback is not system-default support. Native builds on those
  platforms have not been run for this change.
- The current macOS region/clock preferences are exercised without changing OS
  settings. A controlled region/12-hour/24-hour matrix is not yet manually tested.
- Automated GPUI rendering tests are separate from packaged interactive UI and
  VoiceOver acceptance. The running user application is not replaced or restarted.

See [verification](verification.md) for actual final commands and results.

Validation commands invoke the installed standard Cargo directly at
`/opt/homebrew/bin/cargo`; the alternate Cargo on the default PATH produced
same-version GPUI type mismatches. Keep build outputs under a disposable target
rather than overwriting the user's running app:

```sh
python3 scripts/check-native-i18n.test.py
python3 scripts/check-native-i18n.py
/opt/homebrew/bin/cargo fmt --manifest-path crates/monocode-gpui/Cargo.toml --all --check
/opt/homebrew/bin/cargo clippy --locked --offline --manifest-path crates/monocode-gpui/Cargo.toml -p monocode-core -p monocode-gpui --all-targets --profile native-dev --target-dir /private/tmp/monocode-i18n-target -- -D warnings
/opt/homebrew/bin/cargo test --locked --offline --manifest-path crates/monocode-gpui/Cargo.toml -p monocode-core -p monocode-gpui --profile native-dev --target-dir /private/tmp/monocode-i18n-target -- --test-threads=1
/opt/homebrew/bin/cargo build --locked --offline --manifest-path crates/monocode-gpui/Cargo.toml -p monocode-gpui --profile native-dev --target-dir /private/tmp/monocode-i18n-target
```

The test runner needs its normal loopback, file-watching and process-sampling
permissions. No provider calls, commits, pushes, new agent sessions, app
installation or restart are part of these checks.

Review follow-up: automation time controls format only the selected caption
while closed. The existing select callback evaluates the time-option factory
on opening, regenerating labels against current system preferences. There is no
formatter or option cache. Operator `session_id` diagnostics remain raw protocol
errors and must not pass through the UI message catalog.
