#!/usr/bin/env bash
# =============================================================================
# test-salt-api.sh — smoke-test the salt-api HTTPS endpoint
#
# Tests:
#   1. Service health  — nginx + salt-api processes are running
#   2. HTTPS reach     — GET / returns the expected salt-api banner
#   3. Auth (login)    — POST /login returns a token via PAM
#   4. Command exec    — POST /  runs test.ping on '*' using the token
#   5. Logout          — POST /logout invalidates the token
#
# Usage:
#   ./test-salt-api.sh [OPTIONS]
#
# Options:
#   -h HOST        Salt master hostname or IP  (default: 10.10.3.8)
#   -p PORT        nginx HTTPS port            (default: 8443)
#   -u USER        PAM username                (default: jfogar)
#   -w PASSWORD    PAM password                (prompted if omitted; or SALT_API_PASSWORD env var)
#   -c CACERT      Path to mkcert CA cert for TLS verification (or SALT_API_CACERT env var)
#   -k             Skip TLS verification entirely (quick override for home lab)
#   --help         Show this help
# =============================================================================

set -euo pipefail

# ── Colours ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

pass() { echo -e "  ${GREEN}✔${RESET}  $*"; }
fail() { echo -e "  ${RED}✗${RESET}  $*"; FAILURES=$((FAILURES + 1)); }
info() { echo -e "  ${CYAN}→${RESET}  $*"; }
header() { echo -e "\n${BOLD}${YELLOW}$*${RESET}"; }

FAILURES=0

# ── Defaults (match roles/master/defaults/main.yml) ───────────────────────────
HOST="10.10.3.8"
PORT="8443"
USER="jtfogar"
PASSWORD=""
CACERT="${SALT_API_CACERT:-}"   # path to mkcert CA cert; overrides system trust
CURL_TLS_OPT=""

# ── Arg parsing ───────────────────────────────────────────────────────────────
usage() {
  sed -n '/^# Usage:/,/^# =/p' "$0" | sed 's/^# \{0,3\}//'
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h) HOST="$2";    shift 2 ;;
    -p) PORT="$2";    shift 2 ;;
    -u) USER="$2";    shift 2 ;;
    -w) PASSWORD="$2"; shift 2 ;;
    -c) CACERT="$2";  shift 2 ;;
    -k) CURL_TLS_OPT="-k"; CACERT=""; shift ;;
    --help) usage ;;
    *) echo "Unknown option: $1"; usage ;;
  esac
done

# Build TLS option: explicit CA takes priority over -k
[[ -n "$CACERT" ]] && CURL_TLS_OPT="--cacert $CACERT"

BASE_URL="https://${HOST}:${PORT}"

PASSWORD="${SALT_API_PASSWORD:-}"

if [[ -z "$PASSWORD" ]]; then
  read -rsp "Password for ${USER}: " PASSWORD
  echo
fi

# ── Helpers ───────────────────────────────────────────────────────────────────
# Each api_* function writes the response body to $_BODY_TMP and returns
# (echoes) just the HTTP status code. This avoids the subshell variable-
# propagation problem — callers capture status via $() and read the body
# from the temp file.
_BODY_TMP=$(mktemp)
trap "rm -f $_BODY_TMP" EXIT

api_get() {
  curl -sS $CURL_TLS_OPT \
    -o "$_BODY_TMP" \
    -w "%{http_code}" \
    -H 'Accept: application/json' \
    "$BASE_URL$1" 2>>"$_BODY_TMP" || true
}

api_post() {
  local endpoint="$1"; shift
  curl -sS $CURL_TLS_OPT \
    -o "$_BODY_TMP" \
    -w "%{http_code}" \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    "$BASE_URL$endpoint" \
    -d "$@" 2>>"$_BODY_TMP" || true
}

api_post_auth() {
  local endpoint="$1"; local token="$2"; shift 2
  curl -sS $CURL_TLS_OPT \
    -o "$_BODY_TMP" \
    -w "%{http_code}" \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -H "X-Auth-Token: $token" \
    "$BASE_URL$endpoint" \
    -d "$@" 2>>"$_BODY_TMP" || true
}

# ── Test 1 — Service health ───────────────────────────────────────────────────
header "1. Service health (on ${HOST})"
if ssh -o BatchMode=yes -o ConnectTimeout=5 -l "${USER}" "${HOST}" \
    "systemctl is-active --quiet nginx && systemctl is-active --quiet salt-api" 2>/dev/null; then
  pass "nginx and salt-api are active"
else
  fail "One or both services are not running (or SSH unavailable for service check)"
fi

# ── Test 2 — HTTPS reachability ───────────────────────────────────────────────
header "2. HTTPS reachability  ${BASE_URL}/"
HTTP_STATUS=$(api_get "/")
REACH_BODY=$(cat "$_BODY_TMP")

if [[ "$HTTP_STATUS" == "200" ]] && echo "$REACH_BODY" | grep -qi '"return"\|Welcome\|clients'; then
  pass "salt-api banner received (HTTP ${HTTP_STATUS})"
  info "Response: $(echo "$REACH_BODY" | head -c 120)…"
else
  fail "Unexpected response (HTTP ${HTTP_STATUS:-???})"
  info "Response: ${REACH_BODY:-<empty>}"
  if echo "$REACH_BODY" | grep -qi 'SSL\|certificate\|issuer\|verify'; then
    echo -e "\n${YELLOW}Tip: curl cannot verify the mkcert CA. Fix with one of:${RESET}"
    echo -e "  ${CYAN}# Option 1 — skip verification (quick, home-lab only):${RESET}"
    echo    "    $0 -k ..."
    echo -e "  ${CYAN}# Option 2 — trust the mkcert CA (recommended):${RESET}"
    echo    "    scp ${HOST}:/etc/ssl/mkcert-ca/rootCA.pem ./mkcert-ca.pem"
    echo    "    $0 -c ./mkcert-ca.pem ..."
  fi
fi

# ── Test 3 — Auth / login ─────────────────────────────────────────────────────
header "3. Authentication  POST /login"
HTTP_STATUS=$(api_post "/login" \
  "{\"username\": \"${USER}\", \"password\": \"${PASSWORD}\", \"eauth\": \"pam\"}")
LOGIN_RESP=$(cat "$_BODY_TMP")

TOKEN=$(echo "$LOGIN_RESP" | python3 -c \
  "import sys,json; d=json.load(sys.stdin); print(d['return'][0]['token'])" 2>/dev/null || true)

if [[ -n "$TOKEN" ]]; then
  pass "Login successful (HTTP ${HTTP_STATUS}) — token: ${TOKEN:0:16}…"
else
  fail "Login failed (HTTP ${HTTP_STATUS:-???})"
  info "Raw response: ${LOGIN_RESP:-<empty>}"
  if [[ "$HTTP_STATUS" == "401" ]]; then
    echo -e "\n${YELLOW}Tip: 401 means PAM rejected the credentials. Check:${RESET}"
    echo    "  1. Is the user '${USER}' a real Linux account on ${HOST}?"
    echo    "     ssh ${HOST} 'id ${USER}'"
    echo    "  2. Is '${USER}' listed in /etc/salt/master.d/eauth.conf on ${HOST}?"
    echo    "  3. Is the password correct for '${USER}' on ${HOST}?"
    echo    "  Use -u to override the username:  $0 -u <linux-user> ..."
  fi
  echo -e "\n${RED}Cannot continue without a valid token.${RESET}"
  echo "Failures: $FAILURES"
  exit 1
fi

# ── Test 4 — Command execution ────────────────────────────────────────────────
header "4. Command execution  POST /  (test.ping on '*')"
HTTP_STATUS=$(api_post_auth "/" "$TOKEN" \
  '{"client": "local", "tgt": "*", "fun": "test.ping"}')
CMD_RESP=$(cat "$_BODY_TMP")

if echo "$CMD_RESP" | python3 -c \
    "import sys,json; r=json.load(sys.stdin)['return'][0]; assert any(v is True for v in r.values())" \
    2>/dev/null; then
  MINIONS=$(echo "$CMD_RESP" | python3 -c \
    "import sys,json; r=json.load(sys.stdin)['return'][0]; print(', '.join(k for k,v in r.items() if v is True))")
  pass "test.ping returned True for: ${MINIONS}"
else
  fail "test.ping did not return True for any minion (HTTP ${HTTP_STATUS:-???})"
  info "Response: ${CMD_RESP:-<empty>}"
fi

# ── Test 5 — Logout ───────────────────────────────────────────────────────────
header "5. Logout  POST /logout"
HTTP_STATUS=$(api_post_auth "/logout" "$TOKEN" '{}')
LOGOUT_RESP=$(cat "$_BODY_TMP")

if echo "$LOGOUT_RESP" | grep -qi "logout\|success\|return"; then
  pass "Logout acknowledged (HTTP ${HTTP_STATUS})"
else
  fail "Unexpected logout response (HTTP ${HTTP_STATUS:-???})"
  info "Response: ${LOGOUT_RESP:-<empty>}"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo
echo -e "${BOLD}────────────────────────────────────────${RESET}"
if [[ $FAILURES -eq 0 ]]; then
  echo -e "${GREEN}${BOLD}All tests passed.${RESET}"
else
  echo -e "${RED}${BOLD}${FAILURES} test(s) failed.${RESET}"
fi
echo -e "${BOLD}────────────────────────────────────────${RESET}"

exit $FAILURES
