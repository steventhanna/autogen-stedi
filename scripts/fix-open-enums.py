#!/usr/bin/env python3
"""Add an UnknownValue(String) fallback variant to generated string enums.

The Stedi specs declare closed enum lists for many response fields, but the docs on
those same fields warn that payers return non-compliant values (e.g. "Visit" where the
spec only lists "Visits" for UnitForMeasurement). The rust generator emits a closed
Rust enum, so one non-compliant string fails deserialization of the entire response.
This pass rewrites every generated unit-variant string enum to:

  - append `#[serde(untagged)] UnknownValue(String)`, which captures any value not matched
    by a `#[serde(rename = ...)]` variant and round-trips it verbatim on serialize;
  - extend the Display impl with a matching arm;
  - drop `Copy` from the derive list (String is not Copy).

Integer (serde_repr) enums and data-carrying (discriminator/oneOf) enums are skipped.
Guarded by tests/open_enums.rs.

Idempotent: a rewritten enum ends in a non-rename variant, so it no longer matches.

Usage: fix-open-enums.py <src-dir>
"""
import re
import sys
from pathlib import Path

ENUM_RE = re.compile(
    r'#\[derive\((?P<derives>[^)]*)\)\]\n'
    r'pub enum (?P<name>\w+) \{\n'
    r'(?P<body>(?:    #\[serde\(rename = "(?:[^"\\]|\\.)*"\)\]\n    \w+,\n)+)'
    r'(?P<tail>\n?\})'
)

UNKNOWN_VARIANT = (
    '    /// Any value not defined in the spec (payers may return non-compliant values).\n'
    '    #[serde(untagged)]\n'
    '    UnknownValue(String),\n'
)

UNKNOWN_DISPLAY_ARM = '            Self::UnknownValue(s) => write!(f, "{s}"),\n'


def display_re(name: str) -> re.Pattern:
    return re.compile(
        r'(impl std::fmt::Display for ' + re.escape(name) + r' \{\n'
        r'    fn fmt\(&self, f: &mut std::fmt::Formatter\) -> std::fmt::Result \{\n'
        r'        match self \{\n'
        r'(?:            Self::\w+ => write!\(f, "(?:[^"\\]|\\.)*"\),\n)+)'
        r'(        \}\n)'
    )


def process(path: Path) -> int:
    text = path.read_text()
    fixed_names = []

    def rewrite(m: re.Match) -> str:
        derives = m.group('derives')
        # Only plain serde string enums; leave serde_repr integer enums alone.
        if not re.search(r'\bSerialize\b', derives) or not re.search(r'\bDeserialize\b', derives):
            return m.group(0)
        # Specs already use variant names like `Unknown`; `UnknownValue` matches the
        # generator's own apis error-enum convention and collides with nothing today.
        if re.search(r'^    UnknownValue,$', m.group('body'), re.M):
            raise SystemExit(
                f"error: enum {m.group('name')} already has an UnknownValue variant; "
                "pick a new fallback name in scripts/fix-open-enums.py"
            )
        fixed_names.append(m.group('name'))
        derives = re.sub(r'\bCopy, ', '', derives, count=1)
        return (
            f"#[derive({derives})]\n"
            f"pub enum {m.group('name')} {{\n"
            f"{m.group('body')}{UNKNOWN_VARIANT}{m.group('tail')}"
        )

    text = ENUM_RE.sub(rewrite, text)
    for name in fixed_names:
        text, n = display_re(name).subn(r'\g<1>' + UNKNOWN_DISPLAY_ARM + r'\g<2>', text, count=1)
        if n != 1:
            raise SystemExit(f"error: no Display impl found for enum {name} in {path}")
    if fixed_names:
        path.write_text(text)
    return len(fixed_names)


def main() -> None:
    root = Path(sys.argv[1])
    total = 0
    files = 0
    for path in sorted(root.glob('*/models/*.rs')):
        n = process(path)
        if n:
            total += n
            files += 1
    print(f"    open-enums: added UnknownValue fallback to {total} enum(s) in {files} file(s)")


if __name__ == '__main__':
    main()
