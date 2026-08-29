#!/usr/bin/env bash
# Publish the contents of www/ to the gh-pages branch.
#
# Built with plumbing on purpose: the branch is assembled as an object and a ref
# without ever checking anything out, so your working tree, your index and
# whatever you had half-finished are left completely alone.
set -euo pipefail
cd "$(dirname "$0")"

[ -f www/games.wasm ] || { echo "www/games.wasm missing — run ./build.sh first"; exit 1; }

nojekyll=$(printf '' | git hash-object -w --stdin)   # stop Pages running Jekyll
wasm=$(git hash-object -w www/games.wasm)
index=$(git hash-object -w www/index.html)

# git mktree wants entries sorted by name
tree=$(printf '100644 blob %s\t%s\n' \
  "$nojekyll" ".nojekyll" \
  "$wasm"     "games.wasm" \
  "$index"    "index.html" | git mktree)

msg="Deploy $(git rev-parse --short HEAD)"
if parent=$(git rev-parse -q --verify refs/heads/gh-pages); then
  commit=$(git commit-tree "$tree" -p "$parent" -m "$msg")
else
  commit=$(git commit-tree "$tree" -m "$msg")
fi

git update-ref refs/heads/gh-pages "$commit"
git push -q origin gh-pages
echo "published → https://mhdsid.github.io/children-games/"
