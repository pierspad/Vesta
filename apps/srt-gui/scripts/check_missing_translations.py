#!/usr/bin/env python3
"""Audit locale JSON files against English keys.

A key is marked as missing for a locale when:
- the key does not exist in that locale file
- the translation is empty/blank
- the translation text is exactly the same as English
This utility is intentionally kept in the repository for recurring i18n checks.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any


LOCALE_FILE_RE = re.compile(r"^[a-z]{2}\.json$", re.IGNORECASE)


@dataclass
class MissingItem:
    locale: str
    reason: str


def load_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, dict):
        raise ValueError(f"{path} does not contain a JSON object at top-level")
    return data


def normalize_text(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, str):
        return value.strip()
    return str(value).strip()


def audit_locales(locales_dir: Path) -> tuple[dict[str, list[MissingItem]], dict[str, int], list[str]]:
    en_path = locales_dir / "en.json"
    if not en_path.exists():
        raise FileNotFoundError(f"English locale not found: {en_path}")

    en_data = load_json(en_path)
    locale_paths = sorted(
        p
        for p in locales_dir.glob("*.json")
        if p.name != "en.json" and LOCALE_FILE_RE.match(p.name)
    )
    locale_codes = [p.stem for p in locale_paths]

    all_missing: dict[str, list[MissingItem]] = {}
    per_locale_count = {code: 0 for code in locale_codes}

    loaded_locales = {p.stem: load_json(p) for p in locale_paths}

    for key, en_value in en_data.items():
        en_text = normalize_text(en_value)
        for code, data in loaded_locales.items():
            reason: str | None = None
            if key not in data:
                reason = "missing_key"
            else:
                value_text = normalize_text(data.get(key))
                if value_text == "":
                    reason = "empty_value"
                elif value_text == en_text:
                    reason = "same_as_english"

            if reason is not None:
                all_missing.setdefault(key, []).append(MissingItem(locale=code, reason=reason))
                per_locale_count[code] += 1

    return all_missing, per_locale_count, locale_codes


def build_report(locales_dir: Path) -> dict[str, Any]:
    missing_by_key, per_locale_count, locale_codes = audit_locales(locales_dir)

    missing_keys = {}
    for key, issues in sorted(missing_by_key.items()):
        missing_keys[key] = {
            "locales": [item.locale for item in issues],
            "details": [{"locale": item.locale, "reason": item.reason} for item in issues],
        }

    return {
        "locales_dir": str(locales_dir),
        "english_reference": "en.json",
        "total_english_keys": len(load_json(locales_dir / "en.json")),
        "checked_locales": locale_codes,
        "missing_counts_by_locale": per_locale_count,
        "total_keys_with_issues": len(missing_keys),
        "missing_keys": missing_keys,
    }


def main() -> int:
    script_root = Path(__file__).resolve().parents[1]
    default_locales = script_root / "src" / "lib" / "i18n" / "locales"
    default_report = script_root / "reports" / "missing_translations_report.json"

    parser = argparse.ArgumentParser(description="Find missing or untranslated locale keys compared to en.json")
    parser.add_argument(
        "--locales-dir",
        default=str(default_locales),
        help="Directory containing locale JSON files",
    )
    parser.add_argument(
        "--output",
        default=str(default_report),
        help="Output JSON report path",
    )
    args = parser.parse_args()

    locales_dir = Path(args.locales_dir)
    if not locales_dir.exists():
        raise FileNotFoundError(f"Locales directory does not exist: {locales_dir}")

    report = build_report(locales_dir)

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)

    print(f"Report written to: {output_path}")
    print(f"English keys: {report['total_english_keys']}")
    print(f"Keys with issues: {report['total_keys_with_issues']}")
    print("Missing count by locale:")
    for locale, count in sorted(report["missing_counts_by_locale"].items(), key=lambda x: x[0]):
        print(f"  - {locale}: {count}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
