#!/usr/bin/env bash
# Check that all deploy-baba prerequisites are installed and configured.
# Exits 0 if all checks pass, 1 if any check fails.

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

PASS=0
FAIL=0
WARN=0

ok()   { echo -e "  ${GREEN}✓${NC} $*"; PASS=$((PASS + 1)); }
fail() { echo -e "  ${RED}✗${NC} $*"; FAIL=$((FAIL + 1)); }
warn() { echo -e "  ${YELLOW}~${NC} $*"; WARN=$((WARN + 1)); }

echo ""
echo "deploy-baba dev-doctor"
echo "──────────────────────"
echo ""

# ── Rust toolchain ───────────────────────────────────────────────────────────

if command -v rustup &>/dev/null; then
    RUST_VER=$(rustc --version 2>/dev/null | awk '{print $2}')
    ok "rustup present (rustc ${RUST_VER})"
else
    fail "rustup not found — install from https://rustup.rs"
fi

if command -v cargo-lambda &>/dev/null; then
    CL_VER=$(cargo lambda --version 2>/dev/null | awk '{print $2}')
    ok "cargo-lambda ${CL_VER}"
else
    fail "cargo-lambda not found — run: cargo install cargo-lambda"
fi

# ── Node / pnpm ──────────────────────────────────────────────────────────────

if command -v node &>/dev/null; then
    NODE_VER=$(node --version | tr -d 'v')
    NODE_MAJOR=$(echo "${NODE_VER}" | cut -d. -f1)
    if [[ "${NODE_MAJOR}" -ge 20 ]]; then
        ok "node v${NODE_VER} (≥20 required)"
    else
        fail "node v${NODE_VER} — need ≥20 (use nvm: nvm install 20 && nvm use 20)"
    fi
else
    fail "node not found — install via nvm: https://github.com/nvm-sh/nvm"
fi

if command -v pnpm &>/dev/null; then
    PNPM_VER=$(pnpm --version 2>/dev/null)
    PNPM_MAJOR=$(echo "${PNPM_VER}" | cut -d. -f1)
    if [[ "${PNPM_MAJOR}" -ge 8 ]]; then
        ok "pnpm ${PNPM_VER} (≥8 required)"
    else
        fail "pnpm ${PNPM_VER} — need ≥8 (npm install -g pnpm)"
    fi
else
    fail "pnpm not found — run: npm install -g pnpm"
fi

# ── OpenTofu ─────────────────────────────────────────────────────────────────

if command -v tofu &>/dev/null; then
    TOFU_VER=$(tofu version 2>/dev/null | head -1 | awk '{print $2}')
    ok "tofu ${TOFU_VER}"
else
    fail "tofu not found — install from https://opentofu.org/docs/intro/install/"
fi

# ── just ─────────────────────────────────────────────────────────────────────

if command -v just &>/dev/null; then
    JUST_VER=$(just --version 2>/dev/null | awk '{print $2}')
    ok "just ${JUST_VER}"
else
    fail "just not found — run: brew install just"
fi

# ── AWS SSO session ──────────────────────────────────────────────────────────

AWS_PROFILE="${AWS_PROFILE:-deploy-baba}"
if aws sts get-caller-identity --profile "${AWS_PROFILE}" &>/dev/null; then
    ACCOUNT=$(aws sts get-caller-identity --profile "${AWS_PROFILE}" --query Account --output text)
    ok "AWS SSO active (profile: ${AWS_PROFILE}, account: ${ACCOUNT})"
else
    warn "AWS SSO not active for profile '${AWS_PROFILE}' — run: aws sso login --profile ${AWS_PROFILE}"
fi

# ── Agent cache freshness ─────────────────────────────────────────────────────

CACHE_FILE=".agent-cache/index.json"
if [[ -f "${CACHE_FILE}" ]]; then
    CACHE_SHA=$(python3 -c "import json; d=json.load(open('${CACHE_FILE}')); print(d.get('git',{}).get('sha',''))" 2>/dev/null || echo "")
    HEAD_SHA=$(git rev-parse HEAD 2>/dev/null || echo "")
    if [[ "${CACHE_SHA}" == "${HEAD_SHA}" ]]; then
        ok "Agent cache is fresh (SHA: ${HEAD_SHA:0:8})"
    else
        warn "Agent cache is stale (cached: ${CACHE_SHA:0:8}, HEAD: ${HEAD_SHA:0:8}) — run: just cache-refresh"
    fi
else
    warn "Agent cache not found — run: just cache-refresh"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "──────────────────────"
echo -e "  ${GREEN}${PASS} passed${NC}   ${RED}${FAIL} failed${NC}   ${YELLOW}${WARN} warnings${NC}"
echo ""

if [[ ${FAIL} -gt 0 ]]; then
    echo "Fix the failures above before continuing."
    exit 1
else
    echo "All required checks passed."
fi
