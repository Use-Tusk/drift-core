#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/release.sh [patch|minor]
# Default: patch

BUMP_TYPE="${1:-patch}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

if [[ "$BUMP_TYPE" != "patch" && "$BUMP_TYPE" != "minor" ]]; then
  error "Invalid bump type: $BUMP_TYPE. Use 'patch' or 'minor'."
fi

if ! command -v python3 >/dev/null 2>&1; then
  error "python3 is required for version parsing/updating."
fi

if ! command -v node >/dev/null 2>&1; then
  error "node is required for npm package version checks."
fi

if ! command -v npm >/dev/null 2>&1; then
  error "npm is required for Node binding version updates."
fi

info "Bump type: $BUMP_TYPE"

ROOT_CARGO_TOML="Cargo.toml"
PYPROJECT_TOML="bindings/python/pyproject.toml"
NODE_DIR="bindings/node"
NODE_PACKAGE_JSON="$NODE_DIR/package.json"
NODE_PACKAGE_LOCK="$NODE_DIR/package-lock.json"
DEFAULT_BRANCH="main"

read_workspace_version() {
  python3 - <<'PY'
import tomllib
from pathlib import Path

data = tomllib.loads(Path("Cargo.toml").read_text())
print(data["workspace"]["package"]["version"])
PY
}

read_pyproject_version() {
  python3 - <<'PY'
import tomllib
from pathlib import Path

data = tomllib.loads(Path("bindings/python/pyproject.toml").read_text())
print(data["project"]["version"])
PY
}

set_workspace_version() {
  local new_version="$1"
  python3 - "$new_version" <<'PY'
import sys
from pathlib import Path

new_version = sys.argv[1]
path = Path("Cargo.toml")
lines = path.read_text().splitlines()
out = []
in_workspace_package = False
updated = False

for line in lines:
    stripped = line.strip()
    if stripped.startswith("[") and stripped.endswith("]"):
        in_workspace_package = stripped == "[workspace.package]"
    if in_workspace_package and stripped.startswith("version = "):
        indent = line[: len(line) - len(line.lstrip())]
        out.append(f'{indent}version = "{new_version}"')
        updated = True
    else:
        out.append(line)

if not updated:
    raise SystemExit("failed to locate [workspace.package] version in Cargo.toml")

path.write_text("\n".join(out) + "\n")
PY
}

set_pyproject_version() {
  local new_version="$1"
  python3 - "$new_version" <<'PY'
import sys
from pathlib import Path

new_version = sys.argv[1]
path = Path("bindings/python/pyproject.toml")
lines = path.read_text().splitlines()
out = []
in_project = False
updated = False

for line in lines:
    stripped = line.strip()
    if stripped.startswith("[") and stripped.endswith("]"):
        in_project = stripped == "[project]"
    if in_project and stripped.startswith("version = "):
        indent = line[: len(line) - len(line.lstrip())]
        out.append(f'{indent}version = "{new_version}"')
        updated = True
    else:
        out.append(line)

if not updated:
    raise SystemExit("failed to locate [project] version in pyproject.toml")

path.write_text("\n".join(out) + "\n")
PY
}

GH_AVAILABLE=true
if ! command -v gh >/dev/null 2>&1; then
  warn "GitHub CLI (gh) is not installed. You can create release manually."
  GH_AVAILABLE=false
elif ! gh auth status >/dev/null 2>&1; then
  warn "GitHub CLI is not authenticated. You can create release manually."
  GH_AVAILABLE=false
fi

info "Running preflight checks..."

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  error "Not in a git repository."
fi

CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "$DEFAULT_BRANCH" ]]; then
  error "Not on $DEFAULT_BRANCH branch. Current: $CURRENT_BRANCH"
fi

if ! git diff --quiet || ! git diff --staged --quiet; then
  error "Working directory has uncommitted changes. Commit or stash them first."
fi

UNTRACKED=$(git ls-files --others --exclude-standard)
if [[ -n "$UNTRACKED" ]]; then
  warn "Untracked files present (continuing anyway):"
  echo "$UNTRACKED" | head -5
fi

info "Fetching latest from origin..."
git fetch origin "$DEFAULT_BRANCH" --tags

LOCAL_COMMIT=$(git rev-parse HEAD)
REMOTE_COMMIT=$(git rev-parse "origin/$DEFAULT_BRANCH")
if [[ "$LOCAL_COMMIT" != "$REMOTE_COMMIT" ]]; then
  error "Local branch is not up to date with origin/$DEFAULT_BRANCH. Run 'git pull' first."
fi

LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [[ -n "$LAST_TAG" ]]; then
  COMMITS_SINCE_TAG=$(git rev-list "$LAST_TAG"..HEAD --count)
  if [[ "$COMMITS_SINCE_TAG" -eq 0 ]]; then
    error "No commits since last tag ($LAST_TAG). Nothing to release."
  fi
  info "Commits since $LAST_TAG: $COMMITS_SINCE_TAG"
fi

info "Running workspace checks..."
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" cargo check --workspace

CURRENT_WORKSPACE_VERSION=$(read_workspace_version)
CURRENT_PY_VERSION=$(read_pyproject_version)
CURRENT_NODE_VERSION=$(node -p "require('./${NODE_PACKAGE_JSON}').version")

if [[ "$CURRENT_WORKSPACE_VERSION" != "$CURRENT_PY_VERSION" || "$CURRENT_WORKSPACE_VERSION" != "$CURRENT_NODE_VERSION" ]]; then
  error "Version mismatch detected. Cargo=$CURRENT_WORKSPACE_VERSION Python=$CURRENT_PY_VERSION Node=$CURRENT_NODE_VERSION"
fi

info "Current lockstep version: $CURRENT_WORKSPACE_VERSION"

IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_WORKSPACE_VERSION"
if [[ -z "$MAJOR" || -z "$MINOR" || -z "$PATCH" ]]; then
  error "Failed to parse semantic version: $CURRENT_WORKSPACE_VERSION"
fi

case "$BUMP_TYPE" in
  patch)
    PATCH=$((PATCH + 1))
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
NEW_TAG="v${NEW_VERSION}"
info "Version bump: $CURRENT_WORKSPACE_VERSION -> $NEW_VERSION"

if git rev-parse "$NEW_TAG" >/dev/null 2>&1; then
  error "Tag $NEW_TAG already exists."
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Ready to release: $NEW_VERSION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This will:"
echo "  1. Update version in $ROOT_CARGO_TOML to $NEW_VERSION"
echo "  2. Update version in $PYPROJECT_TOML to $NEW_VERSION"
echo "  3. Update version in $NODE_PACKAGE_JSON (+ lockfile) to $NEW_VERSION"
echo "  4. Commit the version bump"
echo "  5. Create and push tag $NEW_TAG"
echo "  6. Optionally create GitHub release via gh CLI"
echo ""

read -r -p "Proceed? [y/N] " REPLY
if [[ ! "$REPLY" =~ ^[Yy]$ ]]; then
  info "Aborted."
  exit 0
fi

info "Updating workspace Cargo version..."
set_workspace_version "$NEW_VERSION"

info "Updating Python package version..."
set_pyproject_version "$NEW_VERSION"

info "Updating Node package version..."
npm version "$NEW_VERSION" --no-git-tag-version --prefix "$NODE_DIR"

UPDATED_WORKSPACE_VERSION=$(read_workspace_version)
UPDATED_PY_VERSION=$(read_pyproject_version)
UPDATED_NODE_VERSION=$(node -p "require('./${NODE_PACKAGE_JSON}').version")
UPDATED_NODE_LOCK_VERSION=$(node -p "require('./${NODE_PACKAGE_LOCK}').version")

if [[ "$UPDATED_WORKSPACE_VERSION" != "$NEW_VERSION" ]]; then
  error "Failed to update workspace Cargo version."
fi
if [[ "$UPDATED_PY_VERSION" != "$NEW_VERSION" ]]; then
  error "Failed to update Python version."
fi
if [[ "$UPDATED_NODE_VERSION" != "$NEW_VERSION" ]]; then
  error "Failed to update Node package version."
fi
if [[ "$UPDATED_NODE_LOCK_VERSION" != "$NEW_VERSION" ]]; then
  error "Failed to update Node lockfile version."
fi

info "Committing version bump..."
git add "$ROOT_CARGO_TOML" "$PYPROJECT_TOML" "$NODE_PACKAGE_JSON" "$NODE_PACKAGE_LOCK"
git commit -m "chore: bump version to $NEW_VERSION"

info "Creating tag $NEW_TAG..."
git tag -a "$NEW_TAG" -m "Release $NEW_VERSION"

info "Pushing commit and tag..."
git push origin "$DEFAULT_BRANCH"
git push origin "$NEW_TAG"

if [[ "$GH_AVAILABLE" == "true" ]]; then
  info "Creating GitHub release..."
  if gh release create "$NEW_TAG" --generate-notes --title "$NEW_TAG"; then
    info "Released $NEW_VERSION"
  else
    warn "Tag was pushed, but GitHub release creation failed."
    warn "Create release manually for tag $NEW_TAG."
  fi
else
  info "Tag $NEW_TAG pushed successfully."
  info "Create GitHub release manually to continue publish workflows."
fi
