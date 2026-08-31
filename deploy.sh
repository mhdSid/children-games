#!/usr/bin/env bash
# Publish the contents of www/ to the gh-pages branch.
#
# Built with plumbing on purpose: the branch is assembled in a throwaway index
# and written straight to a tree, so your working tree, your real index and
# whatever you had half-finished are left completely alone.
set -euo pipefail
cd "$(dirname "$0")"

[ -f www/games.wasm ] || { echo "www/games.wasm missing — run ./build.sh first"; exit 1; }

# Nothing ships that does not parse. A syntax error in a module is invisible
# until someone opens the page, and by then it is already live.
if command -v node >/dev/null 2>&1; then
  for f in www/js/*.js; do
    node --check "$f" || { echo "$f does not parse — not deploying"; exit 1; }
  done
  echo "all modules parse"
fi

# Every import must resolve to a real export, which node --check cannot see.
if command -v node >/dev/null 2>&1; then
  node --input-type=module -e '
    import fs from "fs"
    import path from "path"

    // Names a module actually exports. Has to understand a comma-separated
    // declaration — `export const A = 1, B = 2` exports both, and a checker
    // that only sees the first reports a false failure on the second.
    function exportsOf (src) {
      const names = new Set()
      for (const m of src.matchAll(/export\s+(?:const|let|var)\s+([^\n]+)/g)) {
        for (const d of m[1].matchAll(/([A-Za-z_$][\w$]*)\s*=/g)) names.add(d[1])
      }
      for (const m of src.matchAll(/export\s+(?:async\s+)?function\s*\*?\s*([A-Za-z_$][\w$]*)/g)) names.add(m[1])
      for (const m of src.matchAll(/export\s+class\s+([A-Za-z_$][\w$]*)/g)) names.add(m[1])
      for (const m of src.matchAll(/export\s*{([^}]*)}/g)) {
        for (const part of m[1].split(",")) {
          const as = part.trim().split(/\s+as\s+/)
          const n = (as[1] || as[0] || "").trim()
          if (n) names.add(n)
        }
      }
      return names
    }

    const dir = "www/js"
    let bad = 0
    for (const f of fs.readdirSync(dir).filter(n => n.endsWith(".js"))) {
      const src = fs.readFileSync(path.join(dir, f), "utf8")
      for (const m of src.matchAll(/import\s+([^"\x27]+?)\s+from\s+"\.\/([\w.-]+)"/g)) {
        const target = path.join(dir, m[2])
        if (!fs.existsSync(target)) { console.error(`${f}: missing ${m[2]}`); bad++; continue }
        const has = exportsOf(fs.readFileSync(target, "utf8"))
        const braces = m[1].match(/{([^}]*)}/)
        if (!braces) continue                      // a default import
        for (const part of braces[1].split(",")) {
          const n = part.trim().split(/\s+as\s+/)[0]
          if (n && !has.has(n)) {
            console.error(`${f}: imports ${n} from ${m[2]}, which does not export it`); bad++
          }
        }
      }
    }
    if (bad) { console.error("import graph is broken — not deploying"); process.exit(1) }
    console.log("imports all resolve")
  '
fi

# Assemble the tree in a scratch index so nested paths work and the real index
# is never touched.
tmp_index=$(mktemp -t gh-index)
trap 'rm -f "$tmp_index"' EXIT
GIT_INDEX_FILE="$tmp_index" git read-tree --empty

add() {  # add <path-in-branch> <file-on-disk>
  local sha
  sha=$(git hash-object -w "$2")
  GIT_INDEX_FILE="$tmp_index" git update-index --add --cacheinfo "100644,$sha,$1"
}

add ".nojekyll" /dev/null   # stop Pages running Jekyll
while IFS= read -r f; do
  add "${f#www/}" "$f"
done < <(find www -type f | sort)

tree=$(GIT_INDEX_FILE="$tmp_index" git write-tree)

msg="Deploy $(git rev-parse --short HEAD)"
if parent=$(git rev-parse -q --verify refs/heads/gh-pages); then
  commit=$(git commit-tree "$tree" -p "$parent" -m "$msg")
else
  commit=$(git commit-tree "$tree" -m "$msg")
fi

git update-ref refs/heads/gh-pages "$commit"
git push -q origin gh-pages
echo "published → https://mhdsid.github.io/children-games/"
