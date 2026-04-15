#!/usr/bin/env python3
"""Generate SVG diagrams showing OTAP metrics tables in columnar format.

Produces four diagrams:
  0. Overview of all OTAP metrics table schemas (columns + types only)
  1. Scope attributes, flat metrics (worked example)
  2. Scope attributes as dimensional metrics (worked example)
  3. Data point attributes (worked example)

The worked examples use the "consumer.items" counter metric from the
internal-metrics-sdk design, with 1 dimension (outcome: success /
failed / refused) producing 3 timeseries.

Usage:
    python3 tools/otap-diagram/generate_otap_svg.py

Output goes to tools/otap-diagram/output/*.svg
"""

import os
from dataclasses import dataclass, field

# ── Styling constants ──────────────────────────────────────────────

FONT = "Consolas, 'Courier New', monospace"
FONT_SIZE = 12
HEADER_FONT_SIZE = 11
TITLE_FONT_SIZE = 16
SUBTITLE_FONT_SIZE = 13

CELL_H = 22
CELL_PAD = 8
HEADER_H = 26
TABLE_GAP = 40
TABLE_HGAP = 30
TABLE_TITLE_H = 28
MARGIN = 30
ARROW_MARKER_SIZE = 8

COL_BG = "#f8f9fa"
COL_TABLE_TITLE = "#2c3e50"
COL_TABLE_TITLE_TEXT = "#ffffff"
COL_HEADER_BG = "#34495e"
COL_HEADER_TEXT = "#ecf0f1"
COL_ROW_EVEN = "#ffffff"
COL_ROW_ODD = "#f0f4f8"
COL_BORDER = "#bdc3c7"
COL_TEXT = "#2c3e50"
COL_TYPE_TEXT = "#7f8c8d"
COL_ID_HIGHLIGHT = "#e8f4fd"
COL_FK_LINE = "#3498db"
COL_UNUSED_HEADER = "#7f8c8d"       # muted header for unused columns
COL_UNUSED_DATA = "#ececec"         # grey-ish for unused data cells
COL_SEPARATOR = "#95a5a6"           # vertical separator between groups

SEP_WIDTH = 3                       # width of the visual separator


def _col_is_unused(col) -> bool:
    """A column is unused when it has data rows but every value is empty."""
    if not col.values:
        return False
    return all(str(v).strip() == "" for v in col.values)


@dataclass
class Column:
    name: str
    arrow_type: str
    values: list = field(default_factory=list)
    is_id: bool = False
    is_fk: bool = False
    nullable: bool = False
    width: int = 0


@dataclass
class Table:
    name: str
    payload_type: str
    columns: list
    x: int = 0
    y: int = 0
    width: int = 0
    height: int = 0


def measure_text(text: str) -> int:
    return len(str(text)) * 7 + CELL_PAD * 2


def compute_col_widths(table: Table):
    for col in table.columns:
        header_w = measure_text(col.name)
        type_w = measure_text(col.arrow_type) - CELL_PAD
        val_w = max((measure_text(str(v)) for v in col.values), default=0)
        col.width = max(header_w, type_w, val_w, 60)


def compute_table_size(table: Table):
    # Sort columns: used first, unused last (stable within each group)
    used = [c for c in table.columns if not _col_is_unused(c)]
    unused = [c for c in table.columns if _col_is_unused(c)]
    table.columns = used + unused

    compute_col_widths(table)
    table.width = sum(c.width for c in table.columns)
    if _has_separator(table):
        table.width += SEP_WIDTH
    n_rows = max(len(c.values) for c in table.columns) if table.columns else 0
    table.height = TABLE_TITLE_H + HEADER_H + CELL_H + n_rows * CELL_H


def _has_separator(table: Table) -> bool:
    """True if the table has both used and unused columns."""
    has_used = any(not _col_is_unused(c) for c in table.columns)
    has_unused = any(_col_is_unused(c) for c in table.columns)
    return has_used and has_unused


def xml_escape(s):
    return str(s).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


# ── SVG rendering ──────────────────────────────────────────────────

def render_table(table: Table) -> str:
    parts = []
    x0, y0 = table.x, table.y
    n_rows = max(len(c.values) for c in table.columns) if table.columns else 0
    has_sep = _has_separator(table)

    # Title banner
    parts.append(
        f'<rect x="{x0}" y="{y0}" width="{table.width}" height="{TABLE_TITLE_H}" '
        f'rx="6" ry="6" fill="{COL_TABLE_TITLE}"/>'
    )
    parts.append(
        f'<rect x="{x0}" y="{y0 + TABLE_TITLE_H - 6}" width="{table.width}" '
        f'height="6" fill="{COL_TABLE_TITLE}"/>'
    )
    parts.append(
        f'<text x="{x0 + table.width // 2}" y="{y0 + TABLE_TITLE_H - 8}" '
        f'text-anchor="middle" fill="{COL_TABLE_TITLE_TEXT}" '
        f'font-family="{FONT}" font-size="{HEADER_FONT_SIZE}" font-weight="bold">'
        f'{xml_escape(table.name)}</text>'
    )
    parts.append(
        f'<text x="{x0 + table.width // 2}" y="{y0 + TABLE_TITLE_H - 19}" '
        f'text-anchor="middle" fill="{COL_TYPE_TEXT}" '
        f'font-family="{FONT}" font-size="9" opacity="0.7">'
        f'{xml_escape(table.payload_type)}</text>'
    )

    cy = y0 + TABLE_TITLE_H

    # ── helper: detect the boundary between used and unused columns ──
    def is_sep_boundary(idx):
        """True if the separator should appear before column at idx."""
        if not has_sep or idx == 0:
            return False
        prev_unused = _col_is_unused(table.columns[idx - 1])
        curr_unused = _col_is_unused(table.columns[idx])
        return not prev_unused and curr_unused

    # ── Column headers ──
    cx = x0
    for i, col in enumerate(table.columns):
        if is_sep_boundary(i):
            cx += SEP_WIDTH
        unused = _col_is_unused(col)
        if col.is_id or col.is_fk:
            bg, tc = COL_ID_HIGHLIGHT, COL_TEXT
        elif unused:
            bg, tc = COL_UNUSED_HEADER, COL_HEADER_TEXT
        else:
            bg, tc = COL_HEADER_BG, COL_HEADER_TEXT

        parts.append(
            f'<rect x="{cx}" y="{cy}" width="{col.width}" height="{HEADER_H}" '
            f'fill="{bg}" stroke="{COL_BORDER}" stroke-width="0.5"/>'
        )
        parts.append(
            f'<text x="{cx + col.width // 2}" y="{cy + 11}" '
            f'text-anchor="middle" fill="{tc}" '
            f'font-family="{FONT}" font-size="{HEADER_FONT_SIZE}" font-weight="bold">'
            f'{xml_escape(col.name)}</text>'
        )
        cx += col.width
    cy += HEADER_H

    # ── Type row ──
    cx = x0
    for i, col in enumerate(table.columns):
        if is_sep_boundary(i):
            cx += SEP_WIDTH
        unused = _col_is_unused(col)
        bg = COL_UNUSED_DATA if unused else COL_ROW_ODD
        parts.append(
            f'<rect x="{cx}" y="{cy}" width="{col.width}" height="{CELL_H}" '
            f'fill="{bg}" stroke="{COL_BORDER}" stroke-width="0.5"/>'
        )
        parts.append(
            f'<text x="{cx + col.width // 2}" y="{cy + 15}" '
            f'text-anchor="middle" fill="{COL_TYPE_TEXT}" '
            f'font-family="{FONT}" font-size="10" font-style="italic">'
            f'{xml_escape(col.arrow_type)}</text>'
        )
        cx += col.width
    cy += CELL_H

    # ── Data rows ──
    for row_idx in range(n_rows):
        cx = x0
        base_bg = COL_ROW_EVEN if row_idx % 2 == 0 else COL_ROW_ODD
        for i, col in enumerate(table.columns):
            if is_sep_boundary(i):
                cx += SEP_WIDTH
            unused = _col_is_unused(col)
            if col.is_id or col.is_fk:
                cell_bg = COL_ID_HIGHLIGHT
            elif unused:
                cell_bg = COL_UNUSED_DATA
            else:
                cell_bg = base_bg
            val = col.values[row_idx] if row_idx < len(col.values) else ""
            parts.append(
                f'<rect x="{cx}" y="{cy}" width="{col.width}" height="{CELL_H}" '
                f'fill="{cell_bg}" stroke="{COL_BORDER}" stroke-width="0.5"/>'
            )
            parts.append(
                f'<text x="{cx + col.width // 2}" y="{cy + 15}" '
                f'text-anchor="middle" fill="{COL_TEXT}" '
                f'font-family="{FONT}" font-size="{FONT_SIZE}">'
                f'{xml_escape(val)}</text>'
            )
            cx += col.width
        cy += CELL_H

    # ── Vertical separator between used and unused columns ──
    if has_sep:
        sx = x0
        for i, col in enumerate(table.columns):
            if is_sep_boundary(i):
                break
            sx += col.width
        sep_top = y0 + TABLE_TITLE_H
        sep_bot = y0 + table.height
        parts.append(
            f'<line x1="{sx + SEP_WIDTH // 2}" y1="{sep_top}" '
            f'x2="{sx + SEP_WIDTH // 2}" y2="{sep_bot}" '
            f'stroke="{COL_SEPARATOR}" stroke-width="{SEP_WIDTH}" '
            f'stroke-dasharray="4,3" opacity="0.6"/>'
        )
        parts.append(
            f'<text x="{sx + SEP_WIDTH // 2}" y="{sep_top - 3}" '
            f'text-anchor="middle" fill="{COL_SEPARATOR}" '
            f'font-family="{FONT}" font-size="8" font-style="italic">'
            f'unused</text>'
        )

    # Outline
    parts.append(
        f'<rect x="{x0}" y="{y0}" width="{table.width}" height="{table.height}" '
        f'rx="6" ry="6" fill="none" stroke="{COL_BORDER}" stroke-width="1.5"/>'
    )
    return "\n".join(parts)


def col_center_x(table, col_name):
    """X position for arrow anchors on a named column — offset to the
    right of center so arrows don't overlap the centered text."""
    cx = table.x
    has_sep = _has_separator(table)
    for i, c in enumerate(table.columns):
        if has_sep and i > 0:
            prev_unused = _col_is_unused(table.columns[i - 1])
            curr_unused = _col_is_unused(c)
            if not prev_unused and curr_unused:
                cx += SEP_WIDTH
        if c.name == col_name:
            # Right-of-center: 70% across the column width
            return cx + int(c.width * 0.7)
        cx += c.width
    return table.x + table.width // 2


def col_header_bottom(table):
    """Y of the bottom edge of the column header row."""
    return table.y + TABLE_TITLE_H + HEADER_H


DOT_R = 5          # radius of the connection dot
STUB_LEN = 20      # vertical stub outside the table before the curve


def render_fk_arrow(src_table, src_col_name, dst_table, dst_col_name,
                    label="", color=COL_FK_LINE, offset=0, **kw) -> str:
    """Draw an FK arrow originating from a colored dot on the source
    column header, routing out through the nearest table edge, curving
    to the destination table, and terminating with an arrowhead on a
    colored dot on the destination column header.

    The arrow always exits the source table downward (from the bottom
    edge) and enters the destination table from the top edge, which
    matches the parent→child reading direction of the tables.
    """
    sx = col_center_x(src_table, src_col_name)
    dx = col_center_x(dst_table, dst_col_name)

    # Dot positions: center of column header cell
    src_dot_y = table_col_header_center_y(src_table)
    dst_dot_y = table_col_header_center_y(dst_table)

    # The visible path exits the source table at its bottom edge and
    # enters the destination table at its top edge.
    src_exit_y = src_table.y + src_table.height
    dst_enter_y = dst_table.y

    # Determine whether source is above or below destination and
    # adjust so arrows always route through open space.
    if src_exit_y < dst_enter_y:
        # Normal case: source above destination
        stub_s = src_exit_y + STUB_LEN
        stub_d = dst_enter_y - STUB_LEN
    else:
        # Source is below destination: exit from source top instead
        src_exit_y = src_table.y
        dst_enter_y = dst_table.y + dst_table.height
        stub_s = src_exit_y - STUB_LEN
        stub_d = dst_enter_y + STUB_LEN

    mid_y = (stub_s + stub_d) / 2 + offset

    cid = color.replace("#", "")

    # Path: vertical line inside table (dot → edge), stub, curve,
    # stub, vertical line inside table (edge → dot)
    path = (
        f'M {sx},{src_dot_y} '
        f'L {sx},{src_exit_y} '
        f'L {sx},{stub_s} '
        f'C {sx},{mid_y} {dx},{mid_y} {dx},{stub_d} '
        f'L {dx},{dst_enter_y} '
        f'L {dx},{dst_dot_y} '
    )

    parts = [
        # Colored dot on source column header (open circle = origin)
        f'<circle cx="{sx}" cy="{src_dot_y}" r="{DOT_R}" '
        f'fill="white" stroke="{color}" stroke-width="2"/>',
        # Colored dot on destination column header (filled = target)
        f'<circle cx="{dx}" cy="{dst_dot_y}" r="{DOT_R}" '
        f'fill="{color}"/>',
        # The arrow path
        f'<path d="{path}" fill="none" stroke="{color}" '
        f'stroke-width="1.5" stroke-dasharray="6,3" '
        f'marker-end="url(#ah-{cid})"/>',
    ]
    if label:
        lx = (sx + dx) / 2
        ly = mid_y - 8
        parts.append(
            f'<rect x="{lx - len(label) * 3.5 - 4}" y="{ly - 10}" '
            f'width="{len(label) * 7 + 8}" height="14" rx="3" '
            f'fill="{COL_BG}" opacity="0.9"/>'
        )
        parts.append(
            f'<text x="{lx}" y="{ly}" text-anchor="middle" fill="{color}" '
            f'font-family="{FONT}" font-size="10" font-weight="bold">'
            f'{xml_escape(label)}</text>'
        )
    return "\n".join(parts)


def table_col_header_center_y(table):
    """Y center of the column header row."""
    return table.y + TABLE_TITLE_H + HEADER_H // 2


def arrow_marker(color):
    cid = color.replace("#", "")
    s = ARROW_MARKER_SIZE
    return (
        f'<marker id="ah-{cid}" markerWidth="{s}" markerHeight="{s}" '
        f'refX="{s}" refY="{s // 2}" orient="auto">'
        f'<polygon points="0 0, {s} {s // 2}, 0 {s}" fill="{color}"/></marker>'
    )


def make_svg(tables, fk_arrows, title, subtitle="", note=""):
    for t in tables:
        compute_table_size(t)
    max_x = max(t.x + t.width for t in tables)
    max_y = max(t.y + t.height for t in tables)
    svg_w = max_x + MARGIN * 2
    svg_h = max_y + MARGIN * 2 + 60

    colors = {fk.get("color", COL_FK_LINE) for fk in fk_arrows}
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" '
        f'viewBox="0 0 {svg_w} {svg_h}" width="{svg_w}" height="{svg_h}">',
        '<defs>',
    ]
    for c in colors:
        parts.append(arrow_marker(c))
    parts.append('</defs>')
    parts.append(f'<rect width="{svg_w}" height="{svg_h}" fill="{COL_BG}" rx="10"/>')
    parts.append(
        f'<text x="{svg_w // 2}" y="{MARGIN + 4}" text-anchor="middle" '
        f'fill="{COL_TABLE_TITLE}" font-family="{FONT}" '
        f'font-size="{TITLE_FONT_SIZE}" font-weight="bold">'
        f'{xml_escape(title)}</text>'
    )
    if subtitle:
        parts.append(
            f'<text x="{svg_w // 2}" y="{MARGIN + 22}" text-anchor="middle" '
            f'fill="{COL_TYPE_TEXT}" font-family="{FONT}" '
            f'font-size="{SUBTITLE_FONT_SIZE}">{xml_escape(subtitle)}</text>'
        )
    if note:
        parts.append(
            f'<text x="{svg_w // 2}" y="{svg_h - 12}" text-anchor="middle" '
            f'fill="{COL_TYPE_TEXT}" font-family="{FONT}" font-size="11" '
            f'font-style="italic">{xml_escape(note)}</text>'
        )
    for t in tables:
        parts.append(render_table(t))
    for fk in fk_arrows:
        parts.append(render_fk_arrow(**fk))
    parts.append('</svg>')
    return "\n".join(parts)


# ── Diagram builders ──────────────────────────────────────────────

def make_otlp_tree():
    """Render the same consumer.items example as a nested OTLP protobuf
    tree, showing the row-major structure for contrast with OTAP columns."""

    # ── Nested-box tree renderer (independent of the table renderer) ──

    INDENT = 20        # pixels per nesting level
    LINE_H = 20        # height per text line
    BOX_PAD_X = 12     # horizontal padding inside a box
    BOX_PAD_Y = 6      # vertical padding top/bottom inside a box
    BOX_GAP = 6        # vertical gap between sibling boxes
    # Two alternating greys: lighter for even depth, slightly darker for odd
    GREY_BG = ("#f4f4f4", "#e8e8e8")
    GREY_BORDER = ("#999999", "#777777")
    GREY_TITLE = ("#555555", "#444444")

    class Node:
        """A protobuf message or repeated element in the tree."""
        def __init__(self, msg_type, fields=None, children=None, repeat_label=None):
            self.msg_type = msg_type       # e.g. "NumberDataPoint"
            self.fields = fields or []     # list of (name, value) pairs
            self.children = children or [] # list of Node
            self.repeat_label = repeat_label  # e.g. "[0]"
            # Computed during layout
            self.x = 0
            self.y = 0
            self.w = 0
            self.h = 0

    def layout(node, x, y, avail_w, depth=0):
        """Recursively compute positions and sizes, returns total height."""
        node.x = x
        node.y = y
        node.w = avail_w
        node.depth = depth

        # Header line + field lines
        content_h = BOX_PAD_Y + LINE_H  # header
        content_h += len(node.fields) * LINE_H
        content_h += BOX_PAD_Y  # bottom padding before children

        child_y = y + content_h
        child_w = avail_w - INDENT * 2

        for child in node.children:
            ch = layout(child, x + INDENT, child_y, child_w, depth + 1)
            child_y += ch + BOX_GAP
            content_h += ch + BOX_GAP

        if node.children:
            content_h += BOX_PAD_Y  # extra bottom pad after children

        node.h = content_h
        return content_h

    def render_node(node):
        """Render a single node as nested SVG rectangles."""
        parts = []
        idx = node.depth % 2
        bg = GREY_BG[idx]
        border = GREY_BORDER[idx]
        title_color = GREY_TITLE[idx]

        # Box
        parts.append(
            f'<rect x="{node.x}" y="{node.y}" '
            f'width="{node.w}" height="{node.h}" '
            f'rx="5" ry="5" fill="{bg}" stroke="{border}" stroke-width="1.2"/>'
        )

        ty = node.y + BOX_PAD_Y + 13

        # Header: message type (and optional repeat label)
        label = node.msg_type
        if node.repeat_label:
            label = f'{node.repeat_label} {node.msg_type}'
        parts.append(
            f'<text x="{node.x + BOX_PAD_X}" y="{ty}" '
            f'fill="{title_color}" font-family="{FONT}" '
            f'font-size="12" font-weight="bold">'
            f'{xml_escape(label)}</text>'
        )
        ty += LINE_H

        # Fields
        for fname, fval in node.fields:
            parts.append(
                f'<text x="{node.x + BOX_PAD_X + 8}" y="{ty}" '
                f'fill="{COL_TEXT}" font-family="{FONT}" font-size="11">'
                f'<tspan fill="{COL_TYPE_TEXT}">{xml_escape(fname)}:</tspan> '
                f'{xml_escape(fval)}</text>'
            )
            ty += LINE_H

        # Children
        for child in node.children:
            parts.append(render_node(child))

        return "\n".join(parts)

    # ── Build the tree for the consumer.items example ──

    def make_ndp(idx, outcome, value):
        return Node("NumberDataPoint", [
            ("start_time_unix_nano", "10:00:00"),
            ("time_unix_nano",       "10:00:10"),
            ("as_int",               str(value)),
            ("flags",                "0"),
            ("attributes",           f'[{{outcome: "{outcome}"}}]'),
        ], repeat_label=f'[{idx}]')

    tree = Node("ExportMetricsServiceRequest", children=[
        Node("ResourceMetrics", repeat_label="[0]", children=[
            Node("Resource", [
                ("dropped_attributes_count", "0"),
            ]),
            Node("ScopeMetrics", repeat_label="[0]", children=[
                Node("InstrumentationScope", [
                    ("name",    '"otap"'),
                    ("version", '"0.1"'),
                    ("attributes", '[{node_id: "node-7"}, {pipeline: "ingest"}]'),
                ]),
                Node("Metric", repeat_label="[0]", fields=[
                    ("name",        '"consumer.items"'),
                    ("unit",        '"{item}"'),
                    ("description", '""'),
                ], children=[
                    Node("Sum", [
                        ("aggregation_temporality", "CUMULATIVE"),
                        ("is_monotonic",            "true"),
                    ], children=[
                        make_ndp(0, "success", 142),
                        make_ndp(1, "failed",  3),
                        make_ndp(2, "refused", 0),
                    ]),
                ]),
            ]),
        ]),
    ])

    # ── Layout and render ──

    tree_w = 720
    layout(tree, MARGIN, MARGIN + 40, tree_w)

    svg_w = tree_w + MARGIN * 2
    svg_h = tree.h + MARGIN * 2 + 60

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" '
        f'viewBox="0 0 {svg_w} {svg_h}" width="{svg_w}" height="{svg_h}">',
        f'<rect width="{svg_w}" height="{svg_h}" fill="{COL_BG}" rx="10"/>',
        f'<text x="{svg_w // 2}" y="{MARGIN + 4}" text-anchor="middle" '
        f'fill="{COL_TABLE_TITLE}" font-family="{FONT}" '
        f'font-size="{TITLE_FONT_SIZE}" font-weight="bold">'
        f'OTLP Protobuf: Row-Major Nested Encoding</text>',
        f'<text x="{svg_w // 2}" y="{MARGIN + 22}" text-anchor="middle" '
        f'fill="{COL_TYPE_TEXT}" font-family="{FONT}" '
        f'font-size="{SUBTITLE_FONT_SIZE}">'
        f'Same consumer.items data \u2014 1 dimension (outcome), 3 data points</text>',
        render_node(tree),
        f'<text x="{svg_w // 2}" y="{svg_h - 12}" text-anchor="middle" '
        f'fill="{COL_TYPE_TEXT}" font-family="{FONT}" font-size="11" '
        f'font-style="italic">'
        f'Each data point is a self-contained message carrying all context; '
        f'attributes and metadata are repeated per point</text>',
        '</svg>',
    ]
    return "\n".join(parts)


def _scope_attrs_cols(values_per_row):
    """Build the standard 9-column attr schema with given row data."""
    pids, keys, types, strs = [], [], [], []
    ints, doubles, bools, bytess, sers = [], [], [], [], []
    for pid, key, typ, strv in values_per_row:
        pids.append(pid)
        keys.append(key)
        types.append(typ)
        strs.append(strv)
        ints.append("")
        doubles.append("")
        bools.append("")
        bytess.append("")
        sers.append("")
    return [
        Column("parent_id", "uint16", pids, is_fk=True),
        Column("key", "string/dict", keys),
        Column("type", "uint8", types),
        Column("str", "string/dict", strs, nullable=True),
        Column("int", "int64", ints, nullable=True),
        Column("double", "float64", doubles, nullable=True),
        Column("bool", "bool", bools, nullable=True),
        Column("bytes", "binary", bytess, nullable=True),
        Column("ser", "binary", sers, nullable=True),
    ]


def make_encoding1():
    """Encoding 1: Scope attributes, flat metrics."""
    scope_attrs = Table("ScopeAttrs", "SCOPE_ATTRS", _scope_attrs_cols([
        (0, "node_id", "Str", "node-7"),
        (0, "pipeline", "Str", "ingest"),
    ]))
    metrics = Table("UnivariateMetrics", "UNIVARIATE_METRICS", [
        Column("id", "uint16", [0, 1, 2], is_id=True),
        Column("resource.id", "uint16", [0, 0, 0], nullable=True),
        Column("scope.id", "uint16", [0, 0, 0], nullable=True),
        Column("scope.name", "str/dict", ["otap", "otap", "otap"], nullable=True),
        Column("metric_type", "uint8", ["Sum", "Sum", "Sum"]),
        Column("name", "str/dict", [
            "consumed_success", "consumed_failed", "consumed_refused"]),
        Column("unit", "str/dict", ["{item}", "{item}", "{item}"], nullable=True),
        Column("agg_temp", "int32", ["Cum", "Cum", "Cum"], nullable=True),
        Column("is_monotonic", "bool", [True, True, True], nullable=True),
    ])
    ndp = Table("NumberDataPoint", "NUMBER_DATA_POINTS", [
        Column("id", "uint32", [0, 1, 2], is_id=True, nullable=True),
        Column("parent_id", "uint16", [0, 1, 2], is_fk=True),
        Column("start_time", "ts_ns", ["10:00:00", "10:00:00", "10:00:00"], nullable=True),
        Column("time", "ts_ns", ["10:00:10", "10:00:10", "10:00:10"]),
        Column("int_value", "int64", [142, 3, 0], nullable=True),
        Column("double_value", "float64", ["", "", ""], nullable=True),
        Column("flags", "uint32", [0, 0, 0], nullable=True),
    ])

    scope_attrs.x = MARGIN; scope_attrs.y = MARGIN + 40
    compute_table_size(scope_attrs)
    metrics.x = MARGIN; metrics.y = scope_attrs.y + scope_attrs.height + TABLE_GAP + 10
    compute_table_size(metrics)
    ndp.x = MARGIN; ndp.y = metrics.y + metrics.height + TABLE_GAP + 10
    compute_table_size(ndp)

    return make_svg(
        [scope_attrs, metrics, ndp],
        [
            dict(src_table=scope_attrs, src_col_name="parent_id",
                 dst_table=metrics, dst_col_name="scope.id",
                 label="parent_id \u2192 scope.id", color=COL_FK_LINE),
            dict(src_table=ndp, src_col_name="parent_id",
                 dst_table=metrics, dst_col_name="id",
                 label="parent_id \u2192 id", color="#e74c3c"),
        ],
        "OTAP Metrics: Scope Attributes, Flat Metrics",
        "Encoding 1 \u2014 1 dimension (outcome), 3 timeseries, M=1 metric",
        "Total rows: K + 6M = 2 + 6 = 8  (K=2 scope attrs, M=1 metric \u00d7 3 flat variants)",
    )


def make_encoding2():
    """Encoding 2: Scope attributes as dimensional metrics."""
    scope_attrs = Table("ScopeAttrs", "SCOPE_ATTRS", _scope_attrs_cols([
        (0, "node_id",  "Str", "node-7"),
        (0, "pipeline", "Str", "ingest"),
        (0, "outcome",  "Str", "success"),
        (1, "node_id",  "Str", "node-7"),
        (1, "pipeline", "Str", "ingest"),
        (1, "outcome",  "Str", "failed"),
        (2, "node_id",  "Str", "node-7"),
        (2, "pipeline", "Str", "ingest"),
        (2, "outcome",  "Str", "refused"),
    ]))
    metrics = Table("UnivariateMetrics", "UNIVARIATE_METRICS", [
        Column("id", "uint16", [0, 1, 2], is_id=True),
        Column("resource.id", "uint16", [0, 0, 0], nullable=True),
        Column("scope.id", "uint16", [0, 1, 2], nullable=True),
        Column("scope.name", "str/dict", ["otap", "otap", "otap"], nullable=True),
        Column("metric_type", "uint8", ["Sum", "Sum", "Sum"]),
        Column("name", "str/dict", [
            "consumer.items", "consumer.items", "consumer.items"]),
        Column("unit", "str/dict", ["{item}", "{item}", "{item}"], nullable=True),
        Column("agg_temp", "int32", ["Cum", "Cum", "Cum"], nullable=True),
        Column("is_monotonic", "bool", [True, True, True], nullable=True),
    ])
    ndp = Table("NumberDataPoint", "NUMBER_DATA_POINTS", [
        Column("id", "uint32", [0, 1, 2], is_id=True, nullable=True),
        Column("parent_id", "uint16", [0, 1, 2], is_fk=True),
        Column("start_time", "ts_ns", ["10:00:00", "10:00:00", "10:00:00"], nullable=True),
        Column("time", "ts_ns", ["10:00:10", "10:00:10", "10:00:10"]),
        Column("int_value", "int64", [142, 3, 0], nullable=True),
        Column("double_value", "float64", ["", "", ""], nullable=True),
        Column("flags", "uint32", [0, 0, 0], nullable=True),
    ])

    scope_attrs.x = MARGIN; scope_attrs.y = MARGIN + 40
    compute_table_size(scope_attrs)
    metrics.x = MARGIN; metrics.y = scope_attrs.y + scope_attrs.height + TABLE_GAP + 10
    compute_table_size(metrics)
    ndp.x = MARGIN; ndp.y = metrics.y + metrics.height + TABLE_GAP + 10
    compute_table_size(ndp)

    return make_svg(
        [scope_attrs, metrics, ndp],
        [
            dict(src_table=scope_attrs, src_col_name="parent_id",
                 dst_table=metrics, dst_col_name="scope.id",
                 label="parent_id \u2192 scope.id", color=COL_FK_LINE),
            dict(src_table=ndp, src_col_name="parent_id",
                 dst_table=metrics, dst_col_name="id",
                 label="parent_id \u2192 id", color="#e74c3c"),
        ],
        "OTAP Metrics: Scope Attributes as Dimensions",
        "Encoding 2 \u2014 1 dimension (outcome), 3 timeseries, M=1 metric",
        "Total rows: 3(K+1) + 6M = 3(2+1) + 6 = 15  (outcome promoted into scope attrs)",
    )


def make_encoding3():
    """Encoding 3: Data point attributes."""
    scope_attrs = Table("ScopeAttrs", "SCOPE_ATTRS", _scope_attrs_cols([
        (0, "node_id",  "Str", "node-7"),
        (0, "pipeline", "Str", "ingest"),
    ]))
    metrics = Table("UnivariateMetrics", "UNIVARIATE_METRICS", [
        Column("id", "uint16", [0], is_id=True),
        Column("resource.id", "uint16", [0], nullable=True),
        Column("scope.id", "uint16", [0], nullable=True),
        Column("scope.name", "str/dict", ["otap"], nullable=True),
        Column("metric_type", "uint8", ["Sum"]),
        Column("name", "str/dict", ["consumer.items"]),
        Column("unit", "str/dict", ["{item}"], nullable=True),
        Column("agg_temp", "int32", ["Cum"], nullable=True),
        Column("is_monotonic", "bool", [True], nullable=True),
    ])
    ndp = Table("NumberDataPoint", "NUMBER_DATA_POINTS", [
        Column("id", "uint32", [0, 1, 2], is_id=True, nullable=True),
        Column("parent_id", "uint16", [0, 0, 0], is_fk=True),
        Column("start_time", "ts_ns", ["10:00:00", "10:00:00", "10:00:00"], nullable=True),
        Column("time", "ts_ns", ["10:00:10", "10:00:10", "10:00:10"]),
        Column("int_value", "int64", [142, 3, 0], nullable=True),
        Column("double_value", "float64", ["", "", ""], nullable=True),
        Column("flags", "uint32", [0, 0, 0], nullable=True),
    ])
    dp_attrs = Table("NumberDPAttrs", "NUMBER_DP_ATTRS", _scope_attrs_cols([
        (0, "outcome", "Str", "success"),
        (1, "outcome", "Str", "failed"),
        (2, "outcome", "Str", "refused"),
    ]))
    # Override parent_id type to uint32 for dp attrs
    dp_attrs.columns[0] = Column("parent_id", "uint32",
                                  [0, 1, 2], is_fk=True)

    scope_attrs.x = MARGIN; scope_attrs.y = MARGIN + 40
    compute_table_size(scope_attrs)
    metrics.x = MARGIN; metrics.y = scope_attrs.y + scope_attrs.height + TABLE_GAP + 10
    compute_table_size(metrics)
    ndp.x = MARGIN; ndp.y = metrics.y + metrics.height + TABLE_GAP + 10
    compute_table_size(ndp)
    dp_attrs.x = MARGIN; dp_attrs.y = ndp.y + ndp.height + TABLE_GAP + 10
    compute_table_size(dp_attrs)

    return make_svg(
        [scope_attrs, metrics, ndp, dp_attrs],
        [
            dict(src_table=scope_attrs, src_col_name="parent_id",
                 dst_table=metrics, dst_col_name="scope.id",
                 label="parent_id \u2192 scope.id", color=COL_FK_LINE),
            dict(src_table=ndp, src_col_name="parent_id",
                 dst_table=metrics, dst_col_name="id",
                 label="parent_id \u2192 id", color="#e74c3c"),
            dict(src_table=dp_attrs, src_col_name="parent_id",
                 dst_table=ndp, dst_col_name="id",
                 label="parent_id \u2192 id", color="#27ae60"),
        ],
        "OTAP Metrics: Data Point Attributes",
        "Encoding 3 \u2014 1 dimension (outcome), 3 timeseries, M=1 metric",
        "Total rows: K + 7M = 2 + 7 = 9  (outcome as data-point-level attribute)",
    )


def main():
    out_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "output")
    os.makedirs(out_dir, exist_ok=True)

    diagrams = [
        ("otap-metrics-otlp-protobuf.svg", make_otlp_tree),
        ("otap-metrics-encoding1-flat.svg", make_encoding1),
        ("otap-metrics-encoding2-scope-dims.svg", make_encoding2),
        ("otap-metrics-encoding3-dp-attrs.svg", make_encoding3),
    ]
    for name, fn in diagrams:
        path = os.path.join(out_dir, name)
        with open(path, "w") as f:
            f.write(fn())
        print(f"  \u2713 {path}")
    print(f"\nGenerated {len(diagrams)} SVG diagrams in {out_dir}/")


if __name__ == "__main__":
    main()
