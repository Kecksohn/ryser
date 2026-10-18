#!/usr/bin/env bash
# Prints "changed" if any file under src/ or src-tauri/src/ was modified since the
# last prompt (stamp touched by the UserPromptSubmit hook), else "unchanged".
# ponytail: mtime-based, misses pure deletions; switch to a git-diff hash if that matters.
cd "$(dirname "$0")/../.." || exit 1
stamp=.claude/scratch/.prompt-stamp
if [ ! -f "$stamp" ] || [ -n "$(find src src-tauri/src -type f -newer "$stamp" -print -quit)" ]; then
  echo changed
else
  echo unchanged
fi
