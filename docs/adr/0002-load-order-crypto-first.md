# Default load order: crypto → US → A-share

`load_watchlist_combined()` used to load A-shares first and only appended US / crypto when their `watchlist_*.txt` files existed, so the default TUI was A-share-only. We now load all three built-in lists by default, ordered **crypto → US → A-share**, so cryptocurrency is the priority instrument on startup. `crypto_enabled = false` drops crypto from the combined list (order falls back to US → A-share), preserving the existing kill-switch. The per-file override (`watchlist.txt` / `watchlist_us.txt` / `watchlist_crypto.txt`) still wins over the built-in list when present.

**Considered Options**

- (a) Crypto → US → A-share by default, file overrides still apply, `crypto_enabled` gates crypto — **chosen**.
- (b) Keep file-gating; only reorder when files exist — rejected: contradicts "crypto is the default priority instrument"; default TUI would stay A-share-only.
- (c) Crypto-first only when `watchlist_crypto.txt` present — rejected: same as (b), no default behavior change.

**Consequences**

- The default combined watchlist grows from ~10 A-shares to ~10 crypto + ~25 US + ~10 A-share; startup is noisier and depends on OKX reachability by default.
- `selected_code` defaults to the first crypto symbol (BTC-USDT); signal/backtest scope now covers all three markets unless a rule scopes otherwise.
