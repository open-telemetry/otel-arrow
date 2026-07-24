#!/usr/bin/env python3

import glob
import os
import subprocess
import sys

CR = b'\r'
CRLF = b'\r\n'
LF = b'\n'

# Only check git-tracked files so local build artifacts, venvs, perf results,
# etc. are skipped automatically.
_tracked_files = set(
    os.path.normpath(f)
    for f in subprocess.run(
        ['git', 'ls-files'], capture_output=True, text=True, check=True
    ).stdout.splitlines()
)

def sanitycheck(pattern, allow_utf8 = False, allow_eol = (CRLF, LF), indent = 1):
    error_count = 0

    for filename in glob.glob(pattern, recursive=True):
        if not os.path.isfile(filename):
            continue
        if os.path.normpath(filename) not in _tracked_files:
            continue
        with open(filename, 'rb') as file:
            content = file.read()
            error = []
            eol = None
            lineno = 1
            if not content:
                error.append('  Empty file found')
            elif content[-1] != 10: # LF
                error.append('  Missing a blank line before EOF')
            for line in content.splitlines(True):
                if allow_utf8 and lineno == 1 and line.startswith(b'\xef\xbb\xbf'):
                    line = line[3:]
                if any(b == 7 for b in line):
                    error.append('  TAB found at Ln:{} {}'.format(lineno, line))
                if any(b > 127 for b in line):
                    error.append('  Non-ASCII character found at Ln:{} {}'.format(lineno, line))
                if line[-2:] == CRLF:
                    if not eol:
                        eol = CRLF
                    elif eol != CRLF:
                        error.append('  Inconsistent line ending found at Ln:{} {}'.format(lineno, line))
                    line = line[:-2]
                elif line[-1:] == LF:
                    if not eol:
                        eol = LF
                    elif eol != LF:
                        error.append('  Inconsistent line ending found at Ln:{} {}'.format(lineno, line))
                    line = line[:-1]
                elif line[-1:] == CR:
                    error.append('  CR found at Ln:{} {}'.format(lineno, line))
                    line = line[:-1]
                if eol:
                    if eol not in allow_eol:
                        error.append('  Line ending {} not allowed at Ln:{}'.format(eol, lineno))
                        break
                if line.startswith(b' '):
                    spc_count = 0
                    for c in line:
                        if c != 32:
                            break
                        spc_count += 1
                    if not indent or spc_count % indent:
                        error.append('  {} SPC found at Ln:{} {}'.format(spc_count, lineno, line))
                if line[-1:] == b' ' or line[-1:] == b'\t':
                    error.append('  Trailing space found at Ln:{} {}'.format(lineno, line))
                lineno += 1
            if error:
                error_count += 1
                print('{} [FAIL]'.format(filename), file=sys.stderr)
                for msg in error:
                    print(msg, file=sys.stderr)
            else:
                # print('{} [PASS]'.format(filename))
                pass

    return error_count

# Non-ASCII-only guard for Rust sources (issue #3320). Unlike sanitycheck()
# above, this does NOT enforce indentation / EOL / trailing-space rules -- it
# ONLY rejects non-ASCII bytes, so it can be scoped to a tree with pre-existing
# (intentional) style that we do not want to churn.
#
# Escape hatch: both markers are plain ASCII and are honored ONLY when they
# appear inside a real `//` line comment, determined by line_comment_starts(),
# a small Rust lexer that tracks normal strings, raw strings, char literals and
# block comments ACROSS lines. So a marker sitting inside a string literal (even
# a multi-line one, or a line that also contains `//`) does not exempt anything.
# The two markers are intentionally non-nested -- neither is a substring of the
# other.
#   * per line:  a `//` comment containing  sanitycheck: allow-non-ascii-line
#                exempts just that physical line.
#   * per file:  a `//` comment containing  sanitycheck: allow-non-ascii-file
#                exempts the whole file.
# Use the hatch sparingly, for genuinely intentional non-ASCII (e.g. a test
# fixture that must contain a specific code point). Prefer a \u{XXXX} escape in
# string/char literals, which keeps the source ASCII with no behavior change.
ALLOW_NON_ASCII_LINE = b'sanitycheck: allow-non-ascii-line'
ALLOW_NON_ASCII_FILE = b'sanitycheck: allow-non-ascii-file'


def _scan_char(text, i, n):
    """`text[i]` is a `'`. Return the index just past a closing `'` if this is a
    char/byte-char literal, else None (a lifetime/label like `'a`). Mirrors the
    sweeper's lexer so a literal such as `'"'` is consumed whole and its inner
    quote never toggles string state."""
    if i + 1 >= n:
        return None
    if text[i + 1] == '\\':
        j = i + 2
        if j < n and text[j] == 'x':
            j += 3
        elif j < n and text[j] == 'u':
            # Bounded search: a Rust unicode escape is at most `\u{XXXXXX}` (6
            # hex digits), so a closing `}` for a valid escape is within ~8
            # chars of `u`. Bounding the window keeps this O(1) per call and
            # avoids an O(n) scan-to-EOF on a malformed unterminated `\u{`.
            k = text.find('}', j, j + 10)
            j = k + 1 if k != -1 else j + 1
        else:
            j += 1
        return j + 1 if j < n and text[j] == "'" else None
    if i + 2 < n and text[i + 2] == "'":
        return i + 3
    return None


def line_comment_starts(content):
    """Return a dict mapping 1-based line number -> byte offset within that line
    where a `//` line comment begins OUTSIDE any string/char/block-comment.

    A minimal Rust lexer carries string / raw-string / block-comment state
    across physical lines, so `//` inside a (possibly multi-line) string or a
    char literal is not mistaken for a comment. Raw strings and char literals
    are modeled; this is the same lexical shape the sweeper used. Decoded as
    latin-1 so every byte maps 1:1 to a character (offsets stay byte offsets)
    and non-ASCII bytes never look like ASCII structural characters.
    """
    text = content.decode('latin-1')
    starts = {}
    i, n = 0, len(text)
    lineno, line_start = 1, 0
    NORMAL, STRING, RAWSTRING, BLOCK = range(4)
    mode = NORMAL
    raw_hashes = 0
    block_depth = 0
    escaped = False
    while i < n:
        c = text[i]
        if c == '\n':
            lineno += 1
            i += 1
            line_start = i
            escaped = False
            continue
        if mode == NORMAL:
            if c == '/' and i + 1 < n and text[i + 1] == '/':
                starts.setdefault(lineno, i - line_start)
                nl = text.find('\n', i)
                i = n if nl == -1 else nl
                continue
            if c == '/' and i + 1 < n and text[i + 1] == '*':
                mode, block_depth = BLOCK, 1
                i += 2
                continue
            if c == '"':
                mode, escaped = STRING, False
                i += 1
                continue
            if c in 'rbc':  # raw / byte / c string prefixes: r br cr b c
                j = i
                pfx = ''
                while j < n and text[j] in 'brc' and len(pfx) < 2:
                    pfx += text[j]
                    j += 1
                if pfx in ('r', 'br', 'cr'):
                    h = j
                    while h < n and text[h] == '#':
                        h += 1
                    if h < n and text[h] == '"':
                        raw_hashes = h - j
                        mode, i = RAWSTRING, h + 1
                        continue
                if pfx in ('b', 'c') and j < n and text[j] == '"':
                    mode, escaped, i = STRING, False, j + 1
                    continue
                i += 1
                continue
            if c == "'":
                e = _scan_char(text, i, n)
                i = e if e is not None else i + 1
                continue
            i += 1
        elif mode == STRING:
            if escaped:
                escaped = False
            elif c == '\\':
                escaped = True
            elif c == '"':
                mode = NORMAL
            i += 1
        elif mode == RAWSTRING:
            if c == '"' and text[i + 1:i + 1 + raw_hashes] == '#' * raw_hashes:
                mode = NORMAL
                i += 1 + raw_hashes
                continue
            i += 1
        else:  # BLOCK comment (nestable)
            if c == '/' and i + 1 < n and text[i + 1] == '*':
                block_depth += 1
                i += 2
            elif c == '*' and i + 1 < n and text[i + 1] == '/':
                block_depth -= 1
                i += 2
                if block_depth == 0:
                    mode = NORMAL
            else:
                i += 1
    return starts


def scan_nonascii(content):
    """Return a list of error messages for non-ASCII bytes in `content` (raw
    bytes) not covered by an escape-hatch marker. Pure (no I/O) so it can be
    unit-tested; see tools/test_sanitycheck.py.

    A UTF-8 BOM is deliberately NOT special-cased: a byte-order mark is itself
    non-ASCII and unwanted in Rust source, so it is reported like any other
    non-ASCII byte.
    """
    lines = content.splitlines(True)
    comment_at = line_comment_starts(content)
    # Per-file hatch: the marker must appear inside a real `//` comment.
    for lineno, col in comment_at.items():
        if ALLOW_NON_ASCII_FILE in lines[lineno - 1][col:]:
            return []
    errors = []
    for lineno, line in enumerate(lines, 1):
        if line.isascii():
            continue
        col = comment_at.get(lineno)
        if col is not None and ALLOW_NON_ASCII_LINE in line[col:]:
            continue
        printable = line.decode('ascii', 'replace').rstrip('\r\n')
        errors.append('  Non-ASCII character found at Ln:{}: {}'.format(lineno, printable))
    return errors

def nonascii_check(pattern):
    error_count = 0

    for filename in glob.glob(pattern, recursive=True):
        if not os.path.isfile(filename):
            continue
        if os.path.normpath(filename) not in _tracked_files:
            continue
        with open(filename, 'rb') as file:
            content = file.read()
        errors = scan_nonascii(content)
        if errors:
            error_count += 1
            print('{} [FAIL]'.format(filename), file=sys.stderr)
            for msg in errors:
                print(msg, file=sys.stderr)

    return error_count

if __name__ == '__main__':
    retval = 0
    retval += sanitycheck('.github/**/*.md', allow_eol = (LF,))
    retval += sanitycheck('.github/**/*.yml', allow_eol = (LF,), indent = 2)
    retval += sanitycheck('.github/**/*.yaml', allow_eol = (LF,), indent = 2)
    retval += sanitycheck('**/*.md', allow_eol = (LF,))
    retval += sanitycheck('**/*.py', allow_eol = (LF,))
    retval += sanitycheck('**/*.yaml', allow_eol = (LF,), indent = 2)
    retval += sanitycheck('**/*.yml', allow_eol = (LF,), indent = 2)

    # Reject non-ASCII in otap-dataflow Rust source (issue #3320). Scoped to
    # this tree only; the experimental/ Rust tree intentionally keeps Unicode
    # fixtures, and non-.rs assets pulled in via include_str! are out of scope
    # for this .rs check.
    retval += nonascii_check('rust/otap-dataflow/**/*.rs')

    # `retval` is a count of failing files; collapse to 0/1 so a multiple-of-256
    # failure count cannot wrap to a false "success" exit code.
    sys.exit(1 if retval else 0)
