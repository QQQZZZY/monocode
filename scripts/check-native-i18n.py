#!/usr/bin/env python3
"""Check catalog parity and literal UI calls against the English argument contract."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "crates/monocode-gpui"
LOCALES = {"en": "en", "zh-CN": "zh-Hans"}  # catalog locale -> macOS bundle identifier
TOKEN = re.compile(r'//[^\n]*|"(?:[^"\\]|\\.)*"|[()\[\]]')


def call_end(source, start):
    depth = 1
    for token in TOKEN.finditer(source, start):
        if token[0] == "(":
            depth += 1
        elif token[0] == ")":
            depth -= 1
            if depth == 0:
                return token.start()
    raise AssertionError("unclosed message call")


def validate(required, supplied):
    return required <= supplied  # A translation may omit caller arguments.


def direct_arguments(expression):
    """Recognize only &[ ("name", expression), ... ]; not a Rust parser."""
    expression = expression.strip().rstrip(",").strip()
    if not expression.startswith("&[") or not expression.endswith("]"):
        return None
    body = expression[2:-1]
    # These require Rust lexical rules outside this small checker's scope.
    if re.search(r'\br\#*"|\'|/\*|!\s*[({[]', body):
        return None
    stack, names, start = [], set(), 0
    for token in re.finditer(r'//[^\n]*|"(?:[^"\\]|\\.)*"|[(){}\[\],]', body):
        value = token[0]
        if value.startswith(('"', '//')):
            continue
        if value in "({[":
            stack.append(value)
        elif value in ")}]":
            if not stack or stack.pop() != dict(zip(")}]", "({["))[value]:
                return None
        elif value == "," and not stack:
            item = body[start:token.start()].strip()
            match = re.fullmatch(r'\(\s*"(\w+)"\s*,[\s\S]+\)', item)
            if not match:
                return None
            names.add(match[1])
            start = token.end()
    if stack:
        return None
    item = body[start:].strip()
    if item:
        match = re.fullmatch(r'\(\s*"(\w+)"\s*,[\s\S]+\)', item)
        if not match:
            return None
        names.add(match[1])
    return names


def literal_branches(expression):
    """Recognize only simple literal, if/else, and match return shapes."""
    literal = r'"[A-Za-z][A-Za-z0-9_-]*"'
    expression = expression.strip()
    if re.fullmatch(literal, expression):
        return True
    if re.fullmatch(r'if\s+[^{}]+\{\s*' + literal + r'\s*\}\s*else\s*\{\s*' + literal + r'\s*\}', expression):
        return True
    match = re.fullmatch(r'match\s+[^{}]+\{([^{}]*)\}', expression)
    if not match:
        return False
    arms = [arm.strip() for arm in match[1].split(',') if arm.strip()]
    return bool(arms) and all(re.fullmatch(r'[^{}]+=>\s*' + literal, arm) for arm in arms)


def read_catalog(path):
    entries = {}
    key = None
    for line in path.read_text().splitlines():
        match = re.match(r"^([A-Za-z][A-Za-z0-9_-]*) = (.*)$", line)
        if match:
            key, value = match.groups()
            assert key not in entries, f"{path}: duplicate message: {key}"
            entries[key] = set(re.findall(r"\$([A-Za-z_][A-Za-z0-9_]*)", value))
        elif key is not None and line.startswith(" "):
            entries[key].update(re.findall(r"\$([A-Za-z_][A-Za-z0-9_]*)", line))
    return entries


def main():
    locale_dir = ROOT / "locales"
    present = {path.stem for path in locale_dir.glob("*.ftl")}
    missing_locales = LOCALES.keys() - present
    extra_locales = present - LOCALES.keys()
    assert not missing_locales and not extra_locales, (
        f"{locale_dir}: missing locales {sorted(missing_locales)}; "
        f"extra locales {sorted(extra_locales)}"
    )
    catalogs = {locale: read_catalog(locale_dir / f"{locale}.ftl") for locale in LOCALES}
    entries = catalogs["en"]
    for locale, translated in catalogs.items():
        if locale == "en":
            continue
        missing = entries.keys() - translated.keys()
        extra = translated.keys() - entries.keys()
        assert not missing and not extra, (
            f"{locale}.ftl: missing keys {sorted(missing)}; extra keys {sorted(extra)}"
        )
        for key in entries:
            unexpected = translated[key] - entries[key]
            assert not unexpected, f"{locale}.ftl: {key} unexpected parameters {sorted(unexpected)}"
    category_file = ROOT / "src/settings/notification_preferences.rs"
    if category_file.exists():
        source = category_file.read_text()
        table = re.search(r'pub\(crate\) const CATEGORIES:[\s\S]*?=\s*\[([\s\S]*?)\n\];', source)
        assert table, "notification CATEGORIES table not found"
        pairs = re.findall(r'\(\s*"[^"]+"\s*,\s*"([A-Za-z][A-Za-z0-9_-]*)"\s*\)\s*,', table[1])
        remainder = re.sub(r'\(\s*"[^"]+"\s*,\s*"[A-Za-z][A-Za-z0-9_-]*"\s*\)\s*,', '', table[1]).strip()
        assert pairs and not remainder, "unsupported notification CATEGORIES table"
        for key in pairs:
            for locale, catalog in catalogs.items():
                assert key in catalog, f"{category_file}: {locale}.ftl missing {key}"
                assert validate(catalog[key], set()), f"{category_file}: {locale}.ftl {key} missing {catalog[key]}"
    checked = 0
    unchecked = 0
    for path in (ROOT / "src").rglob("*.rs"):
        source = path.read_text()
        for identifier in re.finditer(r'\.(?:id|debug_selector|key_context)\(', source):
            end = call_end(source, identifier.end())
            assert "i18n::" not in source[identifier.end():end], f"{path}: translated technical identifier"
        for match in re.finditer(r'crate::i18n::(text|format|message)\(', source):
            kind = match[1]
            start = match.end()
            end = call_end(source, start)
            # A match/if expression may choose among several literal keys.
            depth = 0
            first_end = end
            for token in re.finditer(r'"(?:[^"\\]|\\.)*"|[(){}\[\],]', source[start:end]):
                if token[0] in ("(", "{", "["):
                    depth += 1
                elif token[0] in (")", "}", "]"):
                    depth -= 1
                elif token[0] == "," and depth == 0:
                    first_end = start + token.start()
                    break
            argument = source[start:first_end]
            keys = [literal[1] for literal in re.finditer(r'"([A-Za-z][A-Za-z0-9_-]*)"', argument)
                    if not argument[literal.end():].lstrip().startswith(("=>", "|"))]
            for key in keys:
                assert key in entries, f"{path}: missing {key} (en.ftl)"
            if kind == "format":
                supplied = direct_arguments(source[first_end + 1:end])
            elif kind == "message" and source[first_end + 1:end].strip().rstrip(",").strip() != "None":
                supplied = None
            else:
                supplied = set()
            if supplied is not None:
                for key in keys:
                    for locale, catalog in catalogs.items():
                        assert validate(catalog[key], supplied), f"{path}: {key} missing {catalog[key] - supplied} ({locale}.ftl)"
                    checked += 1
            if supplied is None or not keys or not literal_branches(argument):
                unchecked += 1
                print(f"Unchecked: {path.relative_to(ROOT)}:{source.count(chr(10), 0, match.start()) + 1} (unsupported/dynamic expression)")
    assert validate({"name"}, {"name", "unused"})
    assert not validate({"missing"}, {"name"})
    counts = ", ".join(f"{locale}: {len(catalog)}" for locale, catalog in catalogs.items())
    print(f"Checked {checked} literal message calls against catalogs ({counts}).")
    print(f"{unchecked} calls explicitly unchecked; this is not a complete Rust parser.")
    print("Dynamic settings/search keys and duration variants are covered by Rust tests.")


if __name__ == "__main__":
    main()
