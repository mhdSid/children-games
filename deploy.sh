#!/usr/bin/env bash
# Publish the contents of www/ to the gh-pages branch.
#
# Built with plumbing on purpose: the branch is assembled as an object and a ref
# without ever checking anything out, so your working tree, your index and
# whatever you had half-finished are left completely alone.
set -euo pipefail
cd "$(dirname "$0")"

[ -f www/games.wasm ] || { echo "www/games.wasm missing — run ./build.sh first"; exit 1; }

# Parse the page's module script before publishing. A syntax error in there is
# invisible until someone opens the page in a browser, and by then it is live.
if command -v node >/dev/null 2>&1; then
  node -e '
    const fs = require("fs"), os = require("os"), path = require("path");
    const cp = require("child_process");
    const html = fs.readFileSync("www/index.html", "utf8");
    const m = html.match(/<script type="module">\n([\s\S]*)\n<\/script>/);
    if (!m) { console.error("no module script found in www/index.html"); process.exit(1); }
    const f = path.join(os.tmpdir(), "games-check-" + process.pid + ".mjs");
    fs.writeFileSync(f, m[1]);
    try {
      cp.execFileSync(process.execPath, ["--check", f], { stdio: "inherit" });
      console.log("index.html script parses");
    } catch {
      console.error("index.html script does not parse — not deploying");
      process.exit(1);
    } finally {
      fs.unlinkSync(f);
    }
  '
else
  echo "note: node not found, skipping the syntax check"
fi

# Everything in www/ goes, so adding a file there is all it takes to publish
# it. git mktree wants the entries sorted by name.
{
  printf '100644 blob %s\t%s\n' \
    "$(printf '' | git hash-object -w --stdin)" ".nojekyll"   # stop Pages running Jekyll
  for f in www/*; do
    [ -f "$f" ] || continue
    printf '100644 blob %s\t%s\n' "$(git hash-object -w "$f")" "$(basename "$f")"
  done
} | sort -k2 > /tmp/gh-tree.$$
tree=$(git mktree < /tmp/gh-tree.$$)
rm -f /tmp/gh-tree.$$

msg="Deploy $(git rev-parse --short HEAD)"
if parent=$(git rev-parse -q --verify refs/heads/gh-pages); then
  commit=$(git commit-tree "$tree" -p "$parent" -m "$msg")
else
  commit=$(git commit-tree "$tree" -m "$msg")
fi

git update-ref refs/heads/gh-pages "$commit"
git push -q origin gh-pages
echo "published → https://mhdsid.github.io/children-games/"
