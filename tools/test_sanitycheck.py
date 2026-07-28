#!/usr/bin/env python3
"""Unit tests for the non-ASCII guard in tools/sanitycheck.py (issue #3320).

Run: python3 tools/test_sanitycheck.py
Exits non-zero if any case fails. Uses only the standard library and the pure
scan_nonascii() function (no network access). Non-ASCII inputs are built from
\\x byte escapes so this test file itself stays pure ASCII.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import sanitycheck  # noqa: E402

EM = b'\xe2\x80\x94'   # U+2014 EM DASH, UTF-8
BOM = b'\xef\xbb\xbf'  # UTF-8 byte-order mark

CASES = [
    # (name, content_bytes, expected_error_count)
    ('clean ascii', b'let x = 5; // ok\n', 0),
    ('plain non-ascii comment', b'let x = 5; // ' + EM + b'\n', 1),
    ('per-line hatch in a // comment',
     b'let x = 5; // ' + EM + b' sanitycheck: allow-non-ascii-line\n', 0),
    ('per-line hatch honored after a string literal on the line',
     b'let s = "x"; // ' + EM + b' sanitycheck: allow-non-ascii-line\n', 0),
    ('per-line marker is ignored outside a // comment',
     b'let s = "' + EM + b' sanitycheck: allow-non-ascii-line";\n', 1),
    ('per-line marker inside a string literal that also contains // is ignored',
     b'let s = "// ' + EM + b' sanitycheck: allow-non-ascii-line";\n', 1),
    ('per-file hatch in a // comment',
     b'// sanitycheck: allow-non-ascii-file\nlet s = "' + EM + b'";\n', 0),
    ('per-file marker is ignored outside a // comment',
     b'let s = "sanitycheck: allow-non-ascii-file";\nlet t = "' + EM + b'";\n', 1),
    ('per-file marker inside a string literal that also contains // is ignored',
     b'let s = "// sanitycheck: allow-non-ascii-file";\nlet t = "' + EM + b'";\n', 1),
    ('marker on a multi-line string continuation line is ignored',
     b'let s = "first line\n'
     b'// sanitycheck: allow-non-ascii-line ' + EM + b'\n'
     b'last line";\n', 1),
    ('char literal containing a quote does not desync string tracking',
     b'let q = \'"\'; let s = "// sanitycheck: allow-non-ascii-line ' + EM + b'";\n', 1),
    ('marker inside a raw string is ignored',
     b'let s = r#"// sanitycheck: allow-non-ascii-line ' + EM + b'"#;\n', 1),
    ('hatch honored in a real comment after a raw string on the line',
     b'let s = r#"x"#; // ' + EM + b' sanitycheck: allow-non-ascii-line\n', 0),
    ('unicode-escape char literal is consumed, not treated as a lifetime',
     b"let c = '\\u{2764}'; let s = \"" + EM + b"\";\n", 1),
    ('utf-8 BOM is rejected', BOM + b'fn main() {}\n', 1),
    ('each non-ascii line is counted', b'// ' + EM + b'\n// ' + EM + b'\n', 2),
]


def main():
    failures = 0
    for name, content, expected in CASES:
        got = len(sanitycheck.scan_nonascii(content))
        ok = got == expected
        print('{} {}: expected {}, got {}'.format(
            'PASS' if ok else 'FAIL', name, expected, got))
        if not ok:
            failures += 1
    if failures:
        print('{} test(s) failed'.format(failures))
        return 1
    print('all {} tests passed'.format(len(CASES)))
    return 0


if __name__ == '__main__':
    sys.exit(main())
