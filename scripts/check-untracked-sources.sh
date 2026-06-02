#!/usr/bin/env bash
# Guard against the recurring `git commit -a` pitfall: `-a` stages modifications
# to already-tracked files but NOT new untracked ones, so brand-new source files
# (e.g. a new Svelte component or Rust module) get silently left out of the
# commit — which has repeatedly broken the Docker build in CI.
#
# Fails the commit when untracked, non-ignored files exist under the watched
# source trees. Either `git add` them, or bypass intentionally with
# `git commit --no-verify`.
set -euo pipefail

watched=(frontend-cloud/src controller/src agent/src hoister_shared/src)

untracked=$(git ls-files --others --exclude-standard -- "${watched[@]}")

if [ -n "$untracked" ]; then
    echo "✖ Untracked source files are not staged and will be left out of this commit:"
    echo "$untracked" | sed 's/^/    /'
    echo
    echo "  'git commit -a' does NOT include new files. Stage them with 'git add',"
    echo "  or run 'git commit --no-verify' to commit anyway."
    exit 1
fi
