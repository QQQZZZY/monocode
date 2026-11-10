"""Regression tests for the deliberately limited literal-call checker."""
import contextlib
import importlib.util
import io
import re
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("checker", Path(__file__).with_name("check-native-i18n.py"))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class Contracts(unittest.TestCase):
    def check(self, source, catalog="outer = { $count }\ninner = { $count }\n", categories=None, chinese=None, extra_locale=None, omit_locale=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "locales").mkdir()
            (root / "src/test.rs").write_text(source)
            if categories is not None:
                (root / "src/settings").mkdir()
                (root / "src/settings/notification_preferences.rs").write_text(categories)
            if omit_locale != "en":
                (root / "locales/en.ftl").write_text(catalog)
            if omit_locale != "zh-CN":
                (root / "locales/zh-CN.ftl").write_text(catalog if chinese is None else chinese)
            if extra_locale:
                (root / "locales" / extra_locale).write_text("label = Extra\n")
            previous = checker.ROOT
            checker.ROOT = root
            try:
                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    checker.main()
                return output.getvalue()
            finally:
                checker.ROOT = previous

    def test_nested_count_does_not_satisfy_outer_contract(self):
        with self.assertRaisesRegex(AssertionError, "outer missing"):
            self.check('crate::i18n::format("outer", &[("label", crate::i18n::format("inner", &[("count", "2".into())]))]);')

    def test_valid_nested_contract_allows_unused_label(self):
        result = self.check('crate::i18n::format("outer", &[("count", "1".into()), ("label", crate::i18n::format("inner", &[("count", "2".into())]))]);')
        self.assertIn("Checked 2 literal", result)
        self.assertIn("0 calls explicitly unchecked", result)

    def test_dynamic_arguments_are_explicitly_unchecked(self):
        self.assertIn("1 calls explicitly unchecked", self.check('crate::i18n::format("outer", values);'))

    def test_notification_camel_case_keys_and_contracts(self):
        keys = ["notification-category-agentFinished", "notification-category-agentInput", "notification-category-pullRequests"]
        for key in keys:
            with self.subTest(key=key):
                catalog = f"{key} = {{ $count }}\n"
                self.assertIn("Checked 1 literal", self.check(f'crate::i18n::format("{key}", &[("count", "1".into())]);', catalog))
                with self.assertRaisesRegex(AssertionError, "missing"):
                    self.check(f'crate::i18n::text("{key}");', catalog)
                with self.assertRaisesRegex(AssertionError, "missing"):
                    self.check(f'crate::i18n::text("{key}");', "other = value\n")
                with self.assertRaisesRegex(AssertionError, "duplicate"):
                    self.check("", catalog + catalog)

    def test_message_entrypoint_checks_keys_and_reports_dynamic_arguments(self):
        with self.assertRaisesRegex(AssertionError, "missing missing-key"):
            self.check('crate::i18n::message("missing-key", None);')
        with self.assertRaisesRegex(AssertionError, "missing missing-key"):
            self.check('crate::i18n::message("missing-key", Some(&args));')
        with self.assertRaisesRegex(AssertionError, "outer missing"):
            self.check('crate::i18n::message("outer", None);')
        self.assertIn("1 calls explicitly unchecked", self.check('crate::i18n::message("outer", Some(&args));'))
        self.assertIn("1 calls explicitly unchecked", self.check('crate::i18n::message(key, Some(&args));'))

    def test_real_notification_table_keys_are_checked(self):
        actual = (Path(__file__).resolve().parents[1] / "crates/monocode-gpui/src/settings/notification_preferences.rs").read_text()
        table = re.search(r'pub\(crate\) const CATEGORIES:[\s\S]*?\n\];', actual).group()
        keys = ["notification-category-agentFinished", "notification-category-agentInput", "notification-category-pullRequests"]
        catalog = "".join(f"{key} = label\n" for key in keys + ["notification-category-issues", "notification-category-reminders"])
        self.check("", catalog, table)
        for key in keys:
            with self.subTest(key=key):
                with self.assertRaisesRegex(AssertionError, "missing"):
                    self.check("", catalog.replace(f"{key} = label\n", ""), table)
                with self.assertRaisesRegex(AssertionError, "missing"):
                    self.check("", catalog, table.replace(key, key + "Typo"))
                with self.assertRaisesRegex(AssertionError, "missing"):
                    self.check("", catalog.replace(f"{key} = label", f"{key} = {{ $count }}"), table)
                with self.assertRaisesRegex(AssertionError, r"zh-CN.ftl.*missing keys"):
                    self.check("", catalog, table, chinese=catalog.replace(f"{key} = label\n", ""))
                with self.assertRaisesRegex(AssertionError, r"zh-CN.ftl.*unexpected parameters"):
                    self.check("", catalog, table, chinese=catalog.replace(f"{key} = label", f"{key} = {{ $count }}"))

    def test_catalog_parity_and_duplicates_for_each_locale(self):
        english = "title = Title\nnew-setting = { $name }\n"
        chinese = "title = 标题\nnew-setting = { $name }\n"
        self.assertIn("Checked 1 literal", self.check(
            'crate::i18n::format("new-setting", &[("name", value)]);', english, chinese=chinese))
        for changed, expected in [
            (chinese.replace("title = 标题\n", ""), "missing keys.*title"),
            (chinese + "extra = 多余\n", "extra keys.*extra"),
            (chinese + "title = 重复\n", "zh-CN.ftl.*duplicate message: title"),
        ]:
            with self.subTest(changed=changed), self.assertRaisesRegex(AssertionError, expected):
                self.check("", english, chinese=changed)
        with self.assertRaisesRegex(AssertionError, "en.ftl.*duplicate message: title"):
            self.check("", english + "title = Duplicate\n", chinese=chinese)
        with self.assertRaisesRegex(AssertionError, "extra keys.*title"):
            self.check("", english.replace("title = Title\n", ""), chinese=chinese)
        with self.assertRaisesRegex(AssertionError, "missing keys.*new-key"):
            self.check("", english + "new-key = New\n", chinese=chinese)
        for locale in checker.LOCALES:
            with self.subTest(locale=locale), self.assertRaisesRegex(AssertionError, f"missing locales.*{locale}"):
                self.check("", english, chinese=chinese, omit_locale=locale)
        with self.assertRaisesRegex(AssertionError, "extra locales.*fr"):
            self.check("", english, chinese=chinese, extra_locale="fr.ftl")

    def test_translations_may_omit_but_not_add_required_parameters(self):
        english = "count = { $count ->\n [one] { $text } item\n *[other] { $text } items\n}\n"
        chinese = "count = { $text } 项\n"
        self.assertIn("Checked 1 literal", self.check(
            'crate::i18n::format("count", &[("count", n), ("text", text), ("unused", x)]);',
            english, chinese=chinese))
        with self.assertRaisesRegex(AssertionError, r"zh-CN.ftl.*count unexpected parameters.*missing"):
            self.check("", english, chinese="count = { $text } { $missing }\n")
        with self.assertRaisesRegex(AssertionError, "count missing"):
            self.check('crate::i18n::format("count", &[("text", text)]);', english, chinese=chinese)

    def test_locales_match_embedded_catalogs_and_macos_bundle(self):
        root = Path(__file__).resolve().parents[1]
        i18n = (root / "crates/monocode-gpui/src/i18n.rs").read_text()
        embedded = set(re.findall(r'include_str!\("\.\./locales/([^"/]+)\.ftl"\)', i18n))
        self.assertEqual(embedded, set(checker.LOCALES))
        bundle = (root / "scripts/bundle-native-macos.sh").read_text()
        declaration = re.search(r'<key>CFBundleLocalizations</key><array>(.*?)</array>', bundle).group(1)
        self.assertEqual(re.findall(r'<string>([^<]+)</string>', declaration), list(checker.LOCALES.values()))

    def test_mixed_branches_report_unchecked_and_check_literal(self):
        catalog = "plain = Plain\n"
        for args in ["None", "Some(&args)"]:
            source = f'crate::i18n::message(if flag {{ "plain" }} else {{ dynamic_key }}, {args});'
            self.assertIn("1 calls explicitly unchecked", self.check(source, catalog))
            with self.assertRaisesRegex(AssertionError, "missing plain"):
                self.check(source, "other = Other\n")
        for expression in ['if flag { "plain" } else { "plain" }', 'match flag { true => "plain", false => "plain", }']:
            self.assertIn("0 calls explicitly unchecked", self.check(f'crate::i18n::message({expression}, None);', catalog))

    def test_unsupported_lexical_shapes_are_unchecked(self):
        for expression in ['&[("count", r#"raw"#)]', '&[("count", stringify!(x))]', '&[("count", /* nested */ value)]']:
            self.assertIsNone(checker.direct_arguments(expression))


if __name__ == "__main__":
    unittest.main()
