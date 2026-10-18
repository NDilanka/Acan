#!/bin/sh
# Builds and drives steps files one at a time across every caller (agents included) via a shared lock.
# Usage: ./run-steps.sh [concepts/<name>.steps.json ...]   (no args: every steps file)
# Env passes through to drive.mjs (AUDIT=1, SIZE=WxH). OUTROOT sets the shot root (default /tmp/steps).
cd "$(dirname "$0")" || exit 2
[ $# -eq 0 ] && set -- concepts/*.steps.json
root="${OUTROOT:-/tmp/steps}"
fail=0
for f in "$@"; do
  name=$(basename "$f" .steps.json)
  id=${name%%-*}
  log="$root/$name.log"
  mkdir -p "$root/$name"
  flock /tmp/acan-drive.lock sh -c "OUT=live-$id.html node build.mjs $id >/dev/null && node drive.mjs live-$id.html c=$id '$f' '$root/$name'" >"$log" 2>&1
  code=$?
  if [ $code -eq 0 ] && tail -n 3 "$log" | grep -q '^OK'; then echo "$name OK"; else echo "$name FAIL ($code): $(grep -m1 -iE 'fail|error|expected' "$log" | cut -c1-200)"; fail=1; fi
done
exit $fail
