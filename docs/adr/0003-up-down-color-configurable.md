# Up/down colors are configurable; default flipped to 涨=green / 跌=red

Up/down (涨/跌) coloring was hardcoded to the Chinese convention — 涨=Red, 跌=Green — in four places: `pct_color` (watchlist / movers / indices change %), the breadth up/down indicators, and the Buy/Sell side labels (Buy=Red, Sell=Green). We introduce a `ColorScheme` config pair (`up_color` / `down_color`, named colors or `#rrggbb`, parsed safely with fallback) that drives all four surfaces, and set the **default to 涨=green / 跌=red**, overriding the prior hardcoded convention per explicit user request.

**Considered Options**

- (a) One config pair (`up_color` / `down_color`) feeding all four surfaces via a shared lookup — **chosen**: single source of truth, Buy=up_color, Sell=down_color.
- (b) Per-surface config keys — rejected: four keys to keep in sync; users rarely want them different.
- (c) Keep hardcoded Chinese convention — rejected: user explicitly requested the western default.

**Consequences**

- A future reader may be surprised Buy is no longer red; the default is now intentional and config-driven, not a code assumption (see `pct_color` comment update in `src/ui.rs`).
- Invalid color strings fall back to the configured defaults — `load_config` never panics.
