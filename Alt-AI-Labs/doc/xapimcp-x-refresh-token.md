# xapimcp: getting and setting `X_REFRESH_TOKEN`

`xapimcp` runs headless in production, so you must generate the initial refresh token **once** on a laptop (with a browser), then copy it to the server. After that, `xapimcp` refreshes access tokens automatically.

## Prereqs (X developer app)
In the X developer portal for your app:

- Configure an OAuth2 redirect/callback URL:
  - `http://127.0.0.1:8080/callback`
- Ensure your app/user flow supports the scopes:
  - `tweet.read users.read bookmark.read bookmark.write offline.access`

You’ll need:

- `X_CLIENT_ID` (or legacy `X_CONSUMER_KEY`)
- `X_CLIENT_SECRET` (or legacy `X_CONSUMER_SECRET`)

## 1) Generate the refresh token on your laptop
From the repo root (or `Alt-AI-Labs/xapimcp/`), run:

```bash
cd Alt-AI-Labs/xapimcp

# either new names:
export X_CLIENT_ID="..."
export X_CLIENT_SECRET="..."

# or legacy aliases that already exist in your env/service files:
# export X_CONSUMER_KEY="..."
# export X_CONSUMER_SECRET="..."

cargo run --bin authorize-x
```

It will:

- Print an authorization URL
- Start a local callback listener on `http://127.0.0.1:8080/callback`
- After you approve in the browser, print:
  - `X_REFRESH_TOKEN=...`

Copy the printed value.

## 2) Put `X_REFRESH_TOKEN` on the headless server
You have two supported ways to configure it.

### Option A: environment variable (simplest)
Set this in whatever launches `xapimcp` (systemd unit `Environment=`, an env file sourced by the unit, docker env, etc.):

```bash
export X_REFRESH_TOKEN="PASTE_THE_REFRESH_TOKEN_HERE"
```

Also set client credentials (needed for refresh):

```bash
export X_CLIENT_ID="..."
export X_CLIENT_SECRET="..."
```

Restart `xapimcp`.

### Option B: refresh-token file (recommended for refresh-token rotation persistence)
X can rotate refresh tokens. If you only set `X_REFRESH_TOKEN` as an environment variable, the rotated token can’t be persisted by `xapimcp` across restarts.

Instead, write the token to a file and point `xapimcp` at it:

```bash
sudo install -d -m 0700 /etc/xapimcp
printf '%s\n' "PASTE_THE_REFRESH_TOKEN_HERE" | sudo tee /etc/xapimcp/x_refresh_token >/dev/null
sudo chmod 0600 /etc/xapimcp/x_refresh_token

export X_REFRESH_TOKEN_FILE="/etc/xapimcp/x_refresh_token"
```

Notes:

- `xapimcp` reads `X_REFRESH_TOKEN_FILE` first (falling back to `X_REFRESH_TOKEN`).
- On refresh-token rotation, `xapimcp` overwrites the file with the new token.

Restart `xapimcp`.

## 3) Minimal env checklist for `xapimcp`
- `X_CLIENT_ID` + `X_CLIENT_SECRET` (or `X_CONSUMER_KEY` + `X_CONSUMER_SECRET`)
- Either:
  - `X_REFRESH_TOKEN`, **or**
  - `X_REFRESH_TOKEN_FILE`
- Optional:
  - `X_USER_ID` (numeric). If omitted, xapimcp resolves it via `GET /2/users/me` using a valid access token.

## Common failure modes
- **No refresh token returned by `authorize-x`**: your app/scopes likely don’t include `offline.access`, or the app isn’t allowed to mint refresh tokens for that user flow.
- **Server restarts break auth**: use `X_REFRESH_TOKEN_FILE` so rotated refresh tokens persist across restarts.

