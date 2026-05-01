#!/usr/bin/env bash
# CLI demo for Folio — runs each command verbatim from the demo script
# so you can either rehearse start-to-finish or copy-paste blocks during
# recording. Each section pauses for `read` so you control the pace.
#
# Usage:
#   ./demo/scripts/cli-demo.sh
#
# Prerequisites (run once before the demo):
#   cargo build --release -p scribe-cli
#   export FOLIO=$PWD/target/release/scribe-cli   # adjust if needed
#
# Override the binary path with $FOLIO if you keep it elsewhere.

set -e
cd "$(dirname "$0")/../.."

FOLIO="${FOLIO:-./target/release/scribe-cli}"
DEMO_CN="demo/demo-cn.md"
DEMO_EN="demo/demo-en.md"
OUT="demo/outputs"
mkdir -p "$OUT"

pause() {
  echo
  echo "  [press enter to run the next step]"
  read -r _
}

banner() {
  echo
  echo "═══════════════════════════════════════════════════════════════"
  echo "  $*"
  echo "═══════════════════════════════════════════════════════════════"
}

# --- Step 1: version + help -------------------------------------------------
banner "1.  Confirm the binary works"
echo "$ $FOLIO --version"
"$FOLIO" --version
pause

echo "$ $FOLIO --help"
"$FOLIO" --help
pause

# --- Step 2: simplest conversion --------------------------------------------
banner "2.  English Markdown → .docx (default styles)"
echo "$ $FOLIO $DEMO_EN -o $OUT/demo-en-default.docx"
"$FOLIO" "$DEMO_EN" -o "$OUT/demo-en-default.docx"
pause

# --- Step 3: list themes ----------------------------------------------------
banner "3.  List built-in themes"
echo "$ $FOLIO --list-themes"
"$FOLIO" --list-themes
pause

# --- Step 4: each built-in theme -------------------------------------------
banner "4a. English Markdown → 'academic' theme"
echo "$ $FOLIO $DEMO_EN -o $OUT/demo-en-academic.docx --theme academic"
"$FOLIO" "$DEMO_EN" -o "$OUT/demo-en-academic.docx" --theme academic
pause

banner "4b. Chinese Markdown → 'thesis-cn' theme (宋体 + 黑体 + 1.5 倍行距)"
echo "$ $FOLIO $DEMO_CN -o $OUT/demo-cn-thesis.docx --theme thesis-cn"
"$FOLIO" "$DEMO_CN" -o "$OUT/demo-cn-thesis.docx" --theme thesis-cn
pause

banner "4c. English Markdown → 'report' theme (Calibri + blue accents)"
echo "$ $FOLIO $DEMO_EN -o $OUT/demo-en-report.docx --theme report"
"$FOLIO" "$DEMO_EN" -o "$OUT/demo-en-report.docx" --theme report
pause

# --- Step 5: reference-doc inheritance --------------------------------------
banner "5.  Use a real conference template (--reference-doc)"
TEMPLATES_DIR="demo/templates"
if [[ -d "$TEMPLATES_DIR" ]]; then
  for tmpl in "$TEMPLATES_DIR"/*.docx; do
    [[ -f "$tmpl" ]] || continue
    name=$(basename "$tmpl" .docx)
    echo "$ $FOLIO $DEMO_EN -o $OUT/demo-en-${name}.docx --reference-doc $tmpl"
    "$FOLIO" "$DEMO_EN" -o "$OUT/demo-en-${name}.docx" --reference-doc "$tmpl"
  done
else
  echo "  (skip — no templates in demo/templates/, drop a .docx in there to demo this)"
fi
pause

# --- Step 6: error path -----------------------------------------------------
banner "6.  Mutual-exclusion guard rails"
echo "$ $FOLIO $DEMO_EN -o /tmp/x.docx --theme academic --reference-doc demo/templates/whatever.docx"
"$FOLIO" "$DEMO_EN" -o /tmp/x.docx --theme academic --reference-doc demo/templates/whatever.docx 2>&1 || true
pause

banner "Done. Outputs are under $OUT/"
ls -la "$OUT/"
