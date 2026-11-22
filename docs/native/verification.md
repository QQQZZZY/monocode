# Native verification

Status recorded on October 22, 2026, on macOS Apple Silicon. These are local
results, not a release certification. Later dated sections supplement this
snapshot. See [preview limitations](parity.md) for
remaining runtime and platform work.

## Recorded results

| Check | Result |
| --- | --- |
| Complete native check (`npm run native:check`) | Passed |
| Native script tests (`npm run native:test:dev`) | 19 passed, including development restarts, bundle data modes and dependency guards |
| Native Rust formatting and strict Clippy | Passed for core and GPUI, including all targets |
| Native core tests | 547 passed, 1 ignored |
| GPUI tests | 551 passed, 11 ignored |
| Editor asset regeneration and smoke checks | Passed, including large-file diff highlighting |
| Remote checks | 50 core tests, 25 native runtime/UI tests, Connections and secret-input regressions passed; isolated Node-host and loopback SSH fixtures also passed |
| Host suite | 112 passed, 6 skipped |
| Production frontend | 5,560 tests passed, 13 skipped; TypeScript and production build passed under Node 24 |
| Production Rust workspace | 698 tests passed, 1 SSH fixture ignored; formatting and strict Clippy passed |
| Production macOS packaging | Tauri release `.app` built and its local ad-hoc signature verified |
| Native macOS commands | `native:dev` built and launched; `native:build` produced an optimized app without launching it; both bundle signatures verified |
| Native database defaults | Dev opened `Monocode Development/monocode.db`; release `--check-data` read the production database without migration |
| Native UI and chat | Provider order, live Codex streaming, Stop, saved history after relaunch, resumed provider context, and file preview/source passed |
| Native release UI | Restored existing workspace and conversation history from a consistent production database snapshot |
| macOS notifications | Reminder appeared in Notification Center; opening it returned from Settings to the matching chat and cleared the reminder |

The ignored tests require explicit SSH/host fixtures, a Metal device or a manual
frame-cost measurement. Cargo still reports an upstream future-compatibility
advisory for `block 0.1.6`. The preview workflow has not been run as part of these
local checks. Linux/Windows GUI checks and broad live-provider verification
remain outstanding.

Production source, lockfiles and release workflows were checked against `main`;
the native dependency graph remains separate. Live provider requests used the
development database and a temporary project. The release GUI used a disposable
production snapshot with background agents and automations disabled; only the
read-only data check opened the live production database. Notification delivery
used a separate test bundle identity. Notification sounds, Dock badges and
completion/approval alerts were not manually checked.

Reopening Tauri after native use of the live production database remains a manual
check. Concurrent shared-data startup, scheduled tasks and phone connections
remain unverified and deferred.

## Desktop preference migration — October 23, 2026

On Linux, all 27 automated desktop preference tests and strict core Clippy (`--all-targets`,
`--profile native-dev`, `-D warnings`) passed. The installed Linux WebKitGTK
preference store was readable. Synthetic fixtures verified that WebView2 reads
preserve source filenames and contents with the source database both open/locked
and closed, and that empty origins do not hide saved preferences. A WebKitGTK
fixture also verified eight saved projects reach the restored native workspace.
The full core suite passed serially: 557 passed, 1 ignored. The initial parallel
run failed two provider-process fixtures; both passed in a serial rerun and in
the full serial suite.

The Windows-only module's conditional-compilation path passed an isolated
`-D warnings` check on Linux. This was not a native Windows build. Real WebView2
profile compatibility, a Windows build and Windows execution remain unverified.
Copies of actively written LevelDB files are best-effort and may be incomplete
or unreadable; close Tauri before first-launch preference restoration.

The installed-profile integration test is ignored by default because it reads
personal storage. To run that read-only check explicitly:

```sh
cargo test --manifest-path crates/monocode-gpui/Cargo.toml --locked -p monocode-core --profile native-dev installed_desktop_dir_reads -- --ignored
```

## Preview 1 candidate — October 25, 2026

The [native CI run for `b106d32b`](https://github.com/hardbeat920/monocode/actions/runs/40185511105)
passed. It checked core on macOS, Linux and Windows, and formatting, strict
Clippy, tests and a GUI build on macOS. The same revision passed all 657 GPUI
tests locally, with 11 ignored, plus formatting and strict Clippy. Local fixture
tests required access to loopback networking, file watching and process sampling.

A Linux ARM64 source build was launched and exercised in an Ubuntu 24.04 Noodle
VM. Rust 1.98 avoided a compiler failure with Rust 1.99.1, and a VM-specific
shared-memory workaround with software OpenGL enabled rendering. This was a
launch/UI smoke test, not live provider or packaged Linux x64 upgrade coverage.

Packaged preview-to-preview updates, Windows GUI execution, and macOS
completion/approval alerts, sounds and Dock badges still need runtime testing.

## Windows title bar — October 27, 2026

On Windows 10 22H2 x64, a `native-dev` build was launched and its title bar
exercised by hand. The window showed one bar, the app's tab row. Dragging it
moved the window, double-clicking it maximized and restored the window, and
dragging it to the screen's edge snapped the window to half the screen. The
edges and corners resized the window. Minimize, maximize/restore and close
worked. `WM_NCHITTEST` returned the caption for the bar's empty parts, the
client area for tabs and buttons, the caption buttons for the window controls
and the resize edges for the frame.

All 684 GPUI tests ran serially: 668 passed and 16 failed. The same 16 failed
on `f23cb29c` without these changes, in file-link, worktree, terminal and
pane-layout fixtures. Core tests (589 passed, 2 ignored), native script tests
and formatting passed. Strict Clippy failed on existing compiler warnings in
unrelated files of the Windows build.

Client-decorated Linux and the Windows 11 Snap Layouts flyout were not checked.

## Preview 10 candidate — October 29, 2026

The code at [`bc8c73da`](https://github.com/hardbeat920/monocode/commit/bc8c73daa3eeea03171a402ecdca5f1d1f7e89e7)
passed the complete native check locally on macOS Apple Silicon with Rust
1.99.1, matching the toolchain used by the native CI run. This covers the changes
since `native-v0.16.0-preview.9`.

All jobs in the [native preview workflow](https://github.com/hardbeat920/monocode/actions/runs/40900397284)
and [main CI workflow](https://github.com/hardbeat920/monocode/actions/runs/40900397314)
also passed for this commit. Native Core checks passed on macOS, Linux and
Windows; the native desktop formatting, Clippy, tests and build passed on macOS.

| Check | Result |
| --- | --- |
| Native script tests | 29 passed |
| Native workspace formatting | Passed |
| Strict Clippy for core and GPUI, all targets | Passed |
| Native core tests | 640 passed, 2 ignored |
| GPUI tests, run serially | 747 passed, 11 ignored |
| Native macOS build, `native-dev` profile | Passed |

The final checks used:

```sh
RUSTUP_TOOLCHAIN=1.99.1 npm run native:check
cargo +1.99.1 build --manifest-path crates/monocode-gpui/Cargo.toml --locked -p monocode-gpui --profile native-dev
```

Local fixtures required permission to open loopback sockets and inspect
processes.

These results cover automated fixtures and a local development-profile build.
Preview 10 installers, signing/notarization, packaged upgrades, live heartbeat
and background delegation behavior, and live Hermes reasoning-effort behavior
have not been validated by these checks. Linux and Windows runtime coverage
remains as recorded above and in [preview limitations](parity.md).

### Review fixes

The pre-release review fixes retain a Mono's selected permissions for heartbeat
runs, exclude read-only GitHub review lookups from contribution counts, and
resolve commit IDs and line counts in the command's checkout. Regression cases
cover all five permission modes, submitted versus pending reviews, quoted and
repeated `git -C` options, explicit tool working directories and commits batched
across multiple checkouts. The permission fixture also exposed and now covers
a crash when rendering the Mono's usage-limit notice.

Local revalidation with Rust 1.99.1 passed 29 native script tests, 644 core tests
(2 ignored), and 748 GPUI tests (11 ignored). Native workspace formatting,
strict Clippy for core and GPUI across all targets, and the macOS `native-dev`
build passed.
The linked CI runs above cover the `bc8c73da` baseline; these additional review
fixes were checked locally.

The first candidate CI run exposed a timing race in the existing split-pane
animation test: keyboard simulation could finish the animation before the test
froze its elapsed time. The fixture now creates and freezes the split in one
update, retaining its mid-slide and settled-layout assertions. Keyboard split
behavior remains covered by the separate split-and-drag interaction test.

## Preview 11 candidate — October 30, 2026

The release-review fixes based on `dbec0c5f` passed locally on macOS Apple
Silicon with Rust 1.97.1. Branch naming now uses only the selected provider
and account. The ink canvas leases its animation clock after painting has
initialized its state, and releases it when quiet. Legacy empty-session
choices survive both isolated workspace restoration and desktop preference
reconciliation. Changes to the desktop toggle during an upgrade still apply.

Native dependencies were updated to Tokio 1.54.1, rustls-webpki 0.103.16,
webpki-roots 1.0.11, p256 0.14.0 / crypto-bigint 0.7.10, zstd 0.14.3 /
zstd-safe 8.2.0, and pulldown-cmark 0.13.6. The production lockfile was also
updated for the shared mobile-link requirements and TLS patches.

| Check | Result |
| --- | --- |
| Complete native check, including formatting and strict Clippy for all targets | Passed |
| Native script tests | 29 passed |
| Native core tests | 650 passed, 2 ignored |
| GPUI tests, run serially | 761 passed, 11 ignored |
| Shared phone-link tests in the native workspace | 25 passed |
| Shared phone-link tests in the production workspace | 25 passed |
| Native macOS executable build, `native-dev` profile | Passed |
| Native RustSec vulnerability matches | 0; no advisory IDs ignored |
| Long-history CPU benchmark | Passed for preview 10 and this candidate |

Regression coverage includes unsupported/unavailable branch-name providers,
preserving the selected account, first-paint animation startup, settling to
sleep, waking by pointer interaction, reduced motion, and preference
migration with explicit new choices and subsequent desktop edits. A synthetic
identity and enrollment proof generated with p256 0.13.2 verifies that the new
signing library imports the same saved key, preserves its certificate
fingerprint, verifies the old proof, and produces the same proof bytes. The
fixture is public test data, not a real paired device's credentials.

The RustSec database was refreshed to
`2295466c4e9df26647fd67d8fa1c705ac2fc7f81` before the final scan. Eleven
unmaintained-package entries and one informational Rand logger-soundness
warning remain. Rand 0.7.3 comes through the legacy LevelDB reader; its `log`
feature, required to trigger that advisory, is absent from the resolved
all-platform core feature graph. These warnings were not suppressed. This
native result does not clear the separate React/Tauri and npm dependency
graphs: the production Rust lockfile still has seven vulnerability matches
in zip/zstd dependencies, and the npm findings from the review are unchanged.

### Long-history CPU comparison

The ignored `pane_motion_frame_cost_with_long_history` fixture now freezes the
requested animation position. It continues to require one shell render per
sample, allowing a second only when the slide installs its settled layout.
That settling work remains in the measured time; slow samples are not dropped.

An isolated tracked-source copy of preview 10 used the identical measurement
fix, unchanged vendored dependencies, the same compiler, and the same
`native-dev` profile. After one warm-up per revision, three alternating runs
were measured with no compiler jobs active. Each run sampled 80 positions in
a conversation with 10,000 messages and checked that fewer than 100 visible
rows were rendered.

| CPU measurement | Preview 10 | Candidate |
| --- | --- | --- |
| Median of run p50 values | 12.27 ms | 11.87 ms |
| Median of run p95 values | 13.20 ms | 12.72 ms |
| Range of run p95 values | 12.41–13.28 ms | 12.56–13.65 ms |

The overlapping ranges show no material regression in this benchmark; they do
not establish a speedup. These are CPU measurements at simulated 120 Hz
animation positions, not evidence of 120 fps presentation. GPU costs, live
provider contention, canvas power use in a real window, and packaged upgrades
still need runtime checks. The executable was built without launching it or
opening the installed database. Cross-platform CI and signed installers remain
release-pipeline checks.

## Preview 13 candidate — November 3, 2026

Source revision `4550fded` passed the complete native check locally on macOS
Apple Silicon with Rust 1.97.1:

```sh
npm run native:check
```

- Native development, dependency-guard, packaging, and release script tests: 29 passed.
- Native workspace formatting: passed.
- Strict Clippy for core and GPUI, all targets, `native-dev` profile: passed.
- Native core tests: 666 passed, 2 ignored.
- GPUI tests, run serially: 829 passed, 11 ignored.
- Wheel-scroll integration tests: 3 passed.

The first preflight reproduced the latest macOS CI failure: the changes-shelf
fixture assigned `open` after constructing `ChangesShelf::default()`, triggering
`clippy::field_reassign_with_default`. Revision `4550fded` initializes that field
in the struct literal. This changes test setup only. The full successful check
above includes that correction.

The [candidate native workflow](https://github.com/hardbeat920/monocode/actions/runs/41643529160)
checks the same source with Rust 1.99.1. Its core checks passed on macOS, Linux,
and Windows, and the macOS desktop formatting and strict Clippy checks passed.

These local results cover automated fixtures. Signed installers, notarization,
package signatures, and update-feed publication are checked by the native
release workflow. No live-provider requests, companion device tests, or manual
packaged upgrade checks were performed in this preflight. Existing platform
runtime limitations remain as recorded above and in [preview limitations](parity.md).

## Preview 14 candidate — November 5, 2026

Source revision
[`da39885e`](https://github.com/hardbeat920/monocode/commit/da39885e5503a924647bd19b2a2376bc15eea0c0)
passed every native-check stage locally on macOS Apple Silicon with Rust 1.97.1
and Node 26.9.0:

- Native development, dependency-guard, packaging, and release script tests: 29 passed.
- Native workspace formatting: passed.
- Strict Clippy for core and GPUI, all targets, `native-dev` profile: passed.
- Native core tests: 671 passed, 2 ignored.
- GPUI tests, run serially: 843 passed, 11 ignored.
- Wheel-scroll integration tests: 3 passed.

The initial `npm run native:check` passed its script, formatting, and Clippy
stages, then encountered sandbox denials in core fixtures that open loopback
sockets or inspect processes. Rerunning the Rust stage with the required host
permissions completed successfully:

```sh
npm run native:test
```

The machine was also low on disk space during compilation. Removing the older
debug-profile incremental compiler cache made room for the test build; the
`native-dev` artifacts used for this check were retained.

The [candidate native workflow](https://github.com/hardbeat920/monocode/actions/runs/42036909502)
passed all jobs: core checks on macOS, Linux, and Windows, plus the macOS desktop
formatting, strict Clippy, tests, and build. The desktop job completed after the
local results above were first recorded.

The [preview 14 release workflow](https://github.com/hardbeat920/monocode/actions/runs/42040307219)
built all platform packages and passed macOS signing and notarization. Its native
client verifier checked the signed manifest and all six updater packages before
publication. [Preview 14](https://github.com/hardbeat920/monocode/releases/tag/native-v0.16.0-preview.14)
was published on November 5 at 14:34 UTC, with all 14 download and signature
assets uploaded. The public native update feed was checked afterward: it served
`0.16.0-preview.14`, its notes matched the release notes, and all six versioned
updater downloads returned HTTP 200. The GitHub release is marked as a prerelease;
the production channel is unchanged.

These results cover automated fixtures and publication checks. No live-provider
requests, performance benchmarks, companion device tests, or manual packaged
upgrade tests were performed in this preflight. Existing platform runtime
limitations remain as recorded above and in [preview limitations](parity.md).

## Local checks

Run the complete native check from the repository root:

```sh
npm run native:check
```

Core tests run in parallel. GPUI tests run serially to isolate rendering state
and avoid CPU contention with the interpreter's wall-clock limits.

For focused work, run the relevant part:

```sh
npm run native:test:dev
cargo fmt --manifest-path crates/monocode-gpui/Cargo.toml --all --check
cargo clippy --manifest-path crates/monocode-gpui/Cargo.toml --locked -p monocode-core -p monocode-gpui --all-targets --profile native-dev -- -D warnings
npm run native:test
```

On macOS with a Metal device, the renderer regression opens no window:

```sh
cargo test --manifest-path crates/monocode-gpui/Cargo.toml --locked -p monocode-gpui --profile native-dev overlapping_shell_pane -- --ignored
```

## Optional runtime checks

The provider smoke example uses an existing CLI login. `--models` only discovers
the catalog; without it, the example makes two real provider requests and checks
streaming and resumed context:

```sh
cargo run --manifest-path crates/monocode-gpui/Cargo.toml --locked -p monocode-core --profile native-dev --example provider_smoke -- cursor --models
cargo run --manifest-path crates/monocode-gpui/Cargo.toml --locked -p monocode-core --profile native-dev --example provider_smoke -- cursor
```

For long-history interaction, use the isolated fixture from the
[native guide](README.md#checks-and-fixtures). Check typing and scrolling while a
separate real session streams, then switch tabs and verify draft/viewport state,
Stop, and saved history after relaunch. Synthetic history is not sent to providers.

## November 6, 2026 — English-only messages and macOS system formatting

Historical record: the English-only/no-language-setting policy and bundle
declaration in this section were superseded by the November 7 language-setting
and Simplified Chinese implementation. The measurements below remain historical.

The implementation follows [the current specification](internationalization.md),
not the earlier two-language proposals. Only `en.ftl` is embedded; its Fluent
locale is fixed to `en`. The macOS Foundation formatters use the current system
locale independently of UI language. No core/vendor code, persisted language
setting, currency conversion, protocol serialization or time-zone setting was
changed.

Validation on Darwin arm64 used `/opt/homebrew/bin/cargo` with `--locked --offline --profile native-dev` and an isolated target directory at
`/private/tmp/monocode-i18n-target`. The default-PATH alternate Cargo produced
same-version GPUI type mismatches, so it was not used for acceptance. The
individual components of `native:check` were run with this explicit target and
toolchain; the wrapper itself was not invoked.

- Catalog contract check: **2,988 English messages / 3,235 literal calls**;
  duplicate/missing keys and missing caller arguments rejected, extra caller
  arguments permitted. Dynamic settings/search keys are checked by Rust tests.
- `cargo fmt --all --check`, strict Clippy for core/GPUI and all targets, and
  `git diff --check`: passed.
- Full core suite: **671 passed, 0 failed, 2 ignored**.
- Full GPUI suite: **869 passed, 0 failed, 11 ignored**.
- Wheel-scroll integration: **3 passed, 0 failed**; doc-tests had no cases.
- Final targeted rerun after the test-only locale guard: **1 passed**.
- Native development binary: built successfully in the isolated target.
- Native development/packaging scripts: **29 passed**, including English-only
  macOS bundle metadata. Signing/notarization services are mocked in these
  script tests; no app was installed or notarized by this task.
- Temporary release harness importing the production message/formatter modules:
  **14 library tests passed**, including production fallback and **1,139
  original-English copy assertions**. Its probe binary also reran the 13 shared
  module tests successfully; these are duplicate checks, not additional coverage.
  The copy sample is not a proof of every rendered UI state.

Tests cover resource syntax/duplicates, missing keys/arguments, allowed unused
arguments, plural branches, intentional spaces/newlines, system number round
trips and precision/currency, timestamp bounds, schedule captions without
changing stored times, task progress, attachment captions, settings/search and
existing GPUI interaction/layout regressions. The final test-only adjustment
keeps the legacy comma-grouping assertions off macOS; Foundation round-trip
tests cover macOS instead.

A probe against the current machine produced English `Sounds` alongside system
locale `zh_CN`, USD `US$1,234.57` for `1234.567`, and
`2023年11月15日 06:13` for timestamp `1700000000000`. The probe did not change OS
region, hour-cycle or timezone preferences. This is a Foundation/module check,
not packaged UI acceptance. The only build warning is the existing upstream
`block 0.1.6` future-compatibility notice.

The 13 ignored tests remain explicitly unrun: installed-desktop profile reading,
isolated loopback SSH, nine Metal renderer tests, the built Node host round trip,
and a manual CoreText frame-cost measurement. Windows/Linux system formatting
is **not implemented or verified**, and this change was not built on those
platforms. A controlled locale/12-hour/24-hour matrix, interactive candidate UI,
and VoiceOver were **not run**. The running user application and agent sessions
were not restarted, replaced or interrupted. No commits or pushes were made.

Reproduction commands are in [internationalization](internationalization.md).
The full changed-file list, patch (including new files), SHA-256 manifest,
command logs and review notes are retained in
`/private/tmp/monocode-native-i18n-implementation-evidence-2026-11-06.md`.

## November 6, 2026 — targeted independent-review corrections

This historical follow-up superseded the timezone and checker-coverage claims in
the initial record. Its UTC display policy and checker counts are superseded by
the November 7 system-preference follow-up below. The four findings in the
independent review were fixed in the main checkout; Working was not modified or integrated.

- Notes' absolute dates and archived-session dates now use an explicit UTC
  Foundation path. Existing Local date callers are unchanged. Fixed inputs
  `2026-01-01T23:30:00Z` and `2026-01-02T00:30:00Z` are checked against independent
  UTC-midnight epoch constants, not expected values generated by the helper.
- Operator `session_id` again returns raw `Provide {key}` diagnostics. Its
  unused Fluent message was removed. A channel-level regression verifies missing
  `sessionId` returns `Provide sessionId`; wrong types retain the existing earlier
  validator response `sessionId must be an ID`. The helper is tested separately.
- The checker collects only direct argument tuples. Its four Python regression
  tests cover illegal/legal nested calls, dynamic arrays and unsupported lexical
  syntax. The checker then reported **2,987 messages, 3,211 checked literal
  keys/calls and 88 explicitly unchecked calls**; it missed three camel-case
  definitions and the `message` entry point. These counts were not exhaustive.
- The time dropdown uses the existing selection callback with a lazy option
  factory. A GPUI test verifies zero factory calls while constructing the closed
  control; the option generator yields 48 standard or 49 custom-time entries.
  The selected caption still formats each render; opening regenerates options
  with current preferences. No formatter cache or global state was added.

Actual follow-up results: three targeted Rust tests passed; four Python tests
passed; release module harness **15 passed**, including **1,137** retained copy
assertions (the two removed assertions were for the protocol-only message).
Strict GPUI Clippy with all targets, native build, formatting and diff checks
passed. Earlier core/GPUI/integration/script totals above are historical results,
not claimed as rerun in this follow-up. The initial protocol test expectation
incorrectly bypassed the existing validator; it was corrected to match the
actual call chain without changing production validation.

Reproduce with the Cargo flags in [internationalization](internationalization.md):

```sh
python3 scripts/check-native-i18n.test.py
python3 scripts/check-native-i18n.py
# Run cargo test -p monocode-gpui with each filter separately:
# utc_calendar_dates_do_not_cross_local_midnight (historical UTC policy; superseded)
# invalid_session_id_is_a_protocol_error
# closed_time_control_does_not_generate_options
```

Detailed commands/logs, the updated complete patch and source hashes are indexed
in `/private/tmp/monocode-native-i18n-review-fixes-evidence.md`. No app was launched,
installed or restarted. Real dropdown interaction, frame-time benchmarking,
region/hour-cycle matrix, VoiceOver and Windows/Linux remain unverified. The
module-level factory test is not a full rendered-frame or timing measurement.
Independent targeted re-review and Working integration remain with the main
session; neither was performed by this implementation follow-up.

## November 7, 2026 — system preferences and checker keys

This follow-up supersedes the November 6 UTC display policy for notes and archive.
Both fields are update instants: notes write `time::now_ms()` and session storage
writes current milliseconds. Their dates and relative session dates now display
in local time. Date-only values retain their selected business day. The reminder
requests weekday, month, day and system clock without forcing a year. Automation
schedule and template captions use system weekdays and times. Editable habit and
mute times accept system-formatted clocks as well as the existing `HH:mm` input;
stored schedule values and mute timestamps remain unchanged in type and meaning.
USD remains USD with two-place business rounding, without currency conversion.

On this machine, a separate Foundation probe found locale `zh_CN`, timezone
`Asia/Shanghai` (UTC+08:00 at the fixed instant), and a 24-hour clock. The instant
`2026-01-01T23:30:00Z` rendered locally on January 2 at 07:30; the pre-fix notes
and archive tests both rendered January 1 and failed. The corrected actual-call
regressions use two fixed timestamps across UTC midnight and compare against a
separately constructed Foundation formatter for the independently computed local
day. Date-only, system-clock, numeric, percentage and USD tests passed. A separate
`en_US` formatter exercising 12-hour input passed, but the operating-system
region/hour-cycle matrix was not changed or manually accepted. The mute save path
and the GPUI habit create/invalid-input regression passed.

The three camel-case notification keys already existed; their names and stable
category/selector IDs were retained. The checker recognizes legal mixed-case
identifiers, checks literal keys through `text`, `format` and `message`, and
reports dynamic `message` arguments as unchecked. An actual-table Rust test
resolves the notification labels; Python fixtures exercise missing keys,
duplicates, missing arguments and nested tuples. The catalog now has **2,991**
messages: the original 2,990 plus one minute caption. The checker reports **3,205
checked literal calls and 91 unchecked calls**; those numbers are not a coverage
claim, and dynamic expressions remain outside its parser.

Python checker regressions and the catalog check passed. Targeted GPUI tests for
system formatting, instant boundaries, reminder, schedules, template labels,
notification categories, mute input, the habit draft, and the English bundle
passed. GPUI strict Clippy (all targets), `cargo fmt --all --check`, `git diff
--check` and an isolated native development build were run using
`/opt/homebrew/bin/cargo`, `--locked --offline --profile native-dev`, and
`/private/tmp/monocode-i18n-deepseek-target`. The existing `block 0.1.6`
future-incompatibility warning remains.

Windows/Linux system-region integration and native builds, actual OS preference
changes, the region/12-hour/24-hour matrix, packaged interactive UI and VoiceOver
remain unverified. The contribution chart and mute calendar retain their previous
week ordering. Formatter construction is still repeated and was not benchmarked
or cached in this follow-up. The running app was not installed, launched or
restarted; no real provider inference, Working restoration, staging, commit or
push was performed. Incremental patch, source hashes and detailed evidence are in
`/private/tmp/monocode-system-format-key-fixes.md`.

## November 7, 2026 — system-format review follow-up

The previous follow-up's time-input and mute-calendar acceptance was incomplete.
Foundation's non-lenient parser accepted a zero hour in a 12-hour clock, and
mute calendar cells still used Gregorian month boundaries under non-Gregorian
preferences. Time input now verifies the whole consumed input and the formatter's
hour range. Legacy `HH:mm`, valid zero padding, case variants and localized
digits remain accepted; `0:30 PM`, `00:30 AM` and trailing junk are rejected.
The mute grid, day numbers, month navigation and keyboard month changes now use
the current macOS Foundation calendar. The selected local date and stored
schedule/mute values remain unchanged; weekday ordering is unchanged.

Targeted tests passed for 12 AM/PM and invalid input in the actual GPUI habit
creation and mute save paths using a process-only `en_US` preference. A controlled
Islamic calendar test checked the January 2026 cross-month bounds, the actual
mute grid's cell-to-date mapping, previous/next navigation, keyboard month shift
and conversion back to the saved timestamp. System formatter tests also covered
Arabic and Han decimal digits. These controlled tests did not change macOS
preferences and are not packaged interactive acceptance or a complete locale,
calendar or hour-cycle matrix.

The static checker now validates message keys in the actual notification
`CATEGORIES` table for existence and an empty argument contract. It checks
visible literal keys in mixed branches while reporting the unresolved branch as
unchecked. Eight Python regression tests passed, including mutations of all three
actual table keys, missing/duplicate/argument contracts, mixed `message`
branches and the original nested tuple cases. At the recorded shared-worktree
snapshot the checker reported 2,995 messages, 3,211 checked literal calls and 98
unchecked calls; concurrent General/language work contributed resource changes,
so these counts are not this follow-up's change count or a coverage claim.

GPUI strict Clippy with all targets, formatting, diff checks and the isolated
native build passed using `/opt/homebrew/bin/cargo` with `--locked --offline
--profile native-dev` and `/private/tmp/monocode-i18n-deepseek-target`. The
pre-existing `block 0.1.6` future-incompatibility notice remains. No real app,
provider, staging, commit or push was used. Windows/Linux system formatting and
builds, real OS preference switching, packaged UI, VoiceOver and performance
remain unverified. The follow-up patch and logs are indexed in
`/private/tmp/monocode-system-format-key-fixes.md`.

## November 7, 2026 — language settings and multilingual catalog checks

This section supersedes the earlier English-only/no-language-setting policy,
while retaining those records as historical evidence. General now offers System,
English and 简体中文. The app resolves the persisted preference at startup and
only prompts for a restart when the effective language differs. On macOS, system
formatting continues to follow the system region, calendar, time zone and clock
preferences independently of UI language. English and Simplified Chinese use separate Fluent
bundles; Chinese resolution errors retry English. The macOS bundle declares
`en` and `zh-Hans`, and the application embeds `en.ftl` and `zh-CN.ftl`.

The independent second review of the preceding system-format follow-up closed
all four findings within its stated scope; its evidence remains in
`/private/tmp/monocode-system-format-key-fixes-review.md`. This record does not
extend that review to packaged interactive UI or an OS-preference matrix.

In this checker/documentation follow-up, 11 Python tests passed, retaining the
previous eight tests including the real notification `CATEGORIES` mutations,
mixed literal/dynamic branch disclosure and nested tuple regression. New cases
cover per-locale duplicates, missing/extra locales and message IDs, unexpected
translation parameters, allowed omitted parameters and unused caller arguments,
and agreement among the supported locale list, embedded catalogs and macOS
bundle declarations. The checker reported **3,218 literal calls**, **2,995
English and 2,995 Chinese messages**, and **98 explicitly unchecked calls** at
this snapshot. These are recorded observations, not fixed thresholds or proof
of full Rust/Fluent syntax or UI coverage. Actual Fluent parsing, duplicate
rejection, message resolution, fallback and Chinese plural rules are covered by
the Rust tests, not by claiming the Python checker is a full parser.

Using `/opt/homebrew/bin/cargo`, `--locked --offline` and the isolated target
`/private/tmp/monocode-general-language-target`, this follow-up reran 18 i18n
tests, seven language-setting tests, and the isolated Chinese startup/search and
cross-thread test; all passed. Workspace `cargo fmt --all --check`, GPUI binary
Clippy with `-D warnings`, and the GPUI binary build passed. The latter commands
used the current incremental artifacts; they do not imply a full-suite rerun.
The existing `block 0.1.6` future-compatibility and linker `__eh_frame` warnings
remain. Earlier full-suite, locale-matrix and packaging results are not claimed
as rerun here.

The application was not installed, launched or restarted, and no real provider
was called. Actual AppKit window-list behavior, VoiceOver, packaged UI and
Windows/Linux language selection or system formatting remain unverified. Unix
language lookup uses locale environment variables and is not verification of
every desktop environment's native preference mechanism. Logs, scoped snapshots
and the follow-up patch are indexed in
`/private/tmp/monocode-general-language-zh-implementation.md`.

## November 7, 2026 — language review fixes

The four findings in the independent language review were addressed. New and
legacy default session titles are identified from stable stored values, and a
provider switch updates the matching placeholder in storage. A queued settings
save no longer overwrites a first-prompt or manual title. The first-prompt path
still links a pasted PR when the session already has a manual title. Visible
default titles, including sidebar accessibility names and search, use UI
resources; the rename field does not persist its translated placeholder. Inbox
connection headings use the existing localized Connect/Manage messages. The two
Chinese worktree confirmation consequences now describe pending actions.

English and Chinese runs of the in-memory GPUI first-prompt test cover provider
switching, default and legacy placeholders, persisted seed titles, pasted PR
links and manual names. Tests do not call a real provider: the initial title is
the prompt-derived fallback, and the late generated-title application has its own
test. Local and blank-remote conditional title writes are covered without remote
network access; the conditional path refuses a host-bound remote session. Other
direct tests cover the sidebar tree, Inbox headings, search,
renaming, saved drafts and both worktree confirmation toggle states.

Two GPUI test windows generate the real accessibility tree through a
test-support-only activation hook. In short language and searchable theme lists,
keyboard navigation exposed the active option while the selected value stayed
distinct. Enter/Space committed once, Escape cancelled, and Tab/Shift-Tab moved
focus without selecting an unconfirmed option. Filtering remained usable. This
does not claim VoiceOver speech or packaged interactive acceptance.

The 18 Fluent/i18n tests, seven language tests, 11 Python checker regressions,
and five mocked macOS bundle-script cases passed. At this snapshot the checker
found 3,222 literal calls against 2,995 English and 2,995 Chinese messages, with
98 explicitly unchecked dynamic or unsupported calls; this is not a complete
Rust/Fluent parser or a semantic audit of every translation. Native workspace and
test-support vendor formatting, strict Clippy for core and GPUI across all
targets, and the isolated `native-dev` GPUI build passed with standard
`/opt/homebrew/bin/cargo`, `--locked --offline` and
`/private/tmp/monocode-general-language-target`. The existing `block 0.1.6`
future-compatibility and linker `__eh_frame` notices remain.

The running application was not installed, launched or restarted; no real
provider was called. Packaged UI, VoiceOver, other operating systems, and a
complete line-by-line independent review of the Chinese catalog remain outside
these results. The scoped patch, hashes and test logs are recorded in
`/private/tmp/monocode-general-language-zh-review-fixes.md` for independent
review; historical system-format and notification records above remain intact.

## November 10, 2026 — persisted-value boundary and formatter reuse

Closed the follow-up items from the November 7 review. No application was
launched and no provider was called.

- Persisted text is stable again. A habit run stores `ERROR_TIMED_OUT` /
  `ERROR_APP_CLOSED` text and a second-opinion session stores
  `SECOND_OPINION_TITLE`; both are mapped to the current language only when
  displayed. Previously a Chinese UI wrote translated text into `habits.json`
  and the session database, and shipped it to the phone as `lastError` / `runs`.
- Display edges localize stored titles: the Settings → Archive list, reminder
  bodies and non-reminder notification subtitles now use `i18n::session_title`
  instead of the raw stored value.
- Added 9 message keys per locale (session-list empty states, four remote
  send-guard statuses, the remote unsupported-command notice, remote legacy
  recovery, and the attachment limit) and removed the 12 per-locale keys that
  no literal, dynamic or checker reference used.
- `system_format.rs` reuses one decimal, currency, percent and per-template
  date formatter per thread, rebuilt when the system locale identifier changes.
  Every call now sets all properties it depends on, because the formatter is
  shared; a regression test covers cross-call leakage between `integer`,
  `padded_unsigned` and `decimal`.
- The desktop CI job now runs both Python i18n checks before formatting.

Results on macOS Apple Silicon, `/opt/homebrew/bin/cargo`, `--locked`,
`native-dev`:

| Check | Result |
| --- | --- |
| `python3 scripts/check-native-i18n.test.py` | 11 passed |
| `python3 scripts/check-native-i18n.py` | exit 0; 3,247 literal calls; en 3,001 / zh-CN 3,001; 99 explicitly unchecked |
| `cargo fmt --all --check` | passed |
| Strict Clippy, core and GPUI, all targets | passed |
| Core suite | 672 passed, 2 ignored |
| GPUI suite, serial | 928 passed, 0 failed, 11 ignored |
| Wheel-scroll integration | 3 passed |
| Targeted `system_format` tests | 9 passed |

Not covered: the phone still renders `lastError` and `run.outcome` without
localizing them, so stored English text appears as English there; Windows and
Linux system formatting remain unimplemented; no packaged UI or VoiceOver check
was run. The per-frame gain from formatter reuse was not measured end to end —
only the construction cost was known (17.1 µs for a number formatter, 38.0 µs
for a date formatter, against 0.56 / 0.31 µs reused).

## November 10, 2026 — runtime display boundary and test isolation

Closed the user-visible English that still reached the desktop UI in the
November 10 review. The phone was explicitly out of scope for this pass. No
application was launched, no provider was called, and nothing was committed.

- New `src/runtime_text.rs` maps stored runtime text to the current language at
  the display boundary. `conversation::status_label` now delegates to it, and
  the transcript resolves saved `error` rows through it, so a language change
  re-renders existing history. The runtime still writes the same stable English
  into `status`, transcript rows and storage.
- Only user-facing text is mapped. Protocol-consistency diagnostics (a receipt
  for another session or command, a missing run identity) and provider text stay
  raw, as the specification requires; a test asserts the pass-through.
- Localized the git picker's search labels, dialog accessibility names and empty
  states, the orchestration "checkout in use" notice, the SSH port range error
  and the image-picker failure. These were rendering English in a Chinese UI.
- `input::Rejected` carries stable text, so the file editor and notes banners
  resolve it through `runtime_text::editor_rejection` instead of storing a
  translated string.
- Test isolation: `i18n::configure` sets the process-wide `START_LANGUAGE` once,
  so the fourteen tests that called it left every later test thread resolving
  Chinese regardless of order. They now use `configure_thread_for_test`; only
  `chinese_startup_search_and_other_threads`, which deliberately covers startup
  inheritance and `needs_restart`, still calls `configure`.
- Corrected two items from the earlier review: the 42 `settings/keymap.rs`
  command titles are persistence keys and selector IDs, and the keybinding page
  renders the localized `label` instead — they must not be translated. The Mono
  manager's `error` toast carries protocol and storage diagnostics, which the
  specification keeps raw.

Results on macOS Apple Silicon, `/opt/homebrew/bin/cargo`, `--locked`,
`native-dev`, target `/private/tmp/monocode-i18n-target`:

| Check | Result |
| --- | --- |
| `python3 scripts/check-native-i18n.test.py` | 11 passed |
| `python3 scripts/check-native-i18n.py` | exit 0; 3,266 literal calls; en 3,042 / zh-CN 3,042; 101 explicitly unchecked |
| `cargo fmt --all --check` | passed |
| Strict Clippy, core and GPUI, all targets | passed |
| Core suite | 672 passed, 2 ignored |
| GPUI suite, serial | 932 passed, 0 failed, 11 ignored |
| Wheel-scroll integration | 3 passed |

Not covered: the phone is unchanged, so `runtimeModes` labels, Mono status lines
and habit `lastError` text still reach it as desktop English; Windows and Linux
system formatting remain unimplemented; markdown and Mermaid parser messages,
`monos/host.rs` app-protocol errors and editor syntax diagnostics still render
their English text; the working tree remains one large mixed change and has not
been split into reviewable commits; no packaged UI or VoiceOver check was run.
