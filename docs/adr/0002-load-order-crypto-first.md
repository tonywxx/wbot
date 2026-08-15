# Default load order: US → crypto → A-share

`load_watchlist_combined()` loads all markets' watchlists by default, ordered **US → crypto → A-share**, so the US list (read from `watchlist.txt`) is the priority instrument on startup. `crypto_enabled = false` drops crypto from the combined list, preserving the existing kill-switch.

File override rules per market:
- `watchlist.txt` (US) / `watchlist_crypto.txt` (crypto) / `watchlist_a.txt` (A-share): if present and contains at least one ticker, its content wins over the built-in list.
- A file that exists but has no tickers (empty or comment-only) is **skipped** — that market contributes nothing.
- A missing file falls back to the built-in default list.

**Considered Options**

- (a) US → crypto → A-share by default, file overrides still apply, empty file skips the market, `crypto_enabled` gates crypto — **chosen**.
- (b) Keep crypto-first order — rejected: user requested US be the default priority instrument and the first read.

**Consequences**

- The default combined watchlist is ~26 US + ~10 crypto + ~10 A-share; startup depends on OKX reachability only when `crypto_enabled`.
- `selected_code` defaults to the first US symbol (AAPL); signal/backtest scope covers all three markets unless a rule scopes otherwise.
- Supersedes the earlier crypto-first decision recorded in this ADR's original version.
