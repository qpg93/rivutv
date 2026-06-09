# Spider Engine — Design Spec

## Problem

TVBox sources (e.g., 饭太硬) define sites with `type: 3` (Spider plugin). Each such
site requires a custom Spider implementation (`csp_*Guard`) that knows how to fetch,
parse, and return TVBox-format data. The current code only supports type 0/1/2/4
HTTP API sites — type 3 sites show "Spider not yet supported."

## Solution

A **Spider Engine** that:
1. Defines a `Spider` trait with the 5 standard TVBox operations
2. Maintains a registry of Spider implementations keyed by `csp_*` name
3. Provides a unified `SpiderApi` that routes calls to `SiteApi` (type 0/1/2/4)
   or `Spider` (type 3) based on the site's type field
4. Ships with 3 built-in Spider implementations that cover the most popular
   content sources among 饭太硬's 48 type-3 sites

## Architecture

### New/Modified Files

```
crates/rivu-spider/src/
├── lib.rs            # + re-export spider module
├── site_api.rs       # unchanged — HTTP API for type 0/1/2/4
├── engine.rs         # + SpiderApi enum/struct (replaces direct SiteApi use)
└── spider/
    ├── mod.rs        # Spider trait, SpiderRegistry
    ├── bili.rs       # csp_BiliGuard — Bilibili public API
    ├── ygp.rs        # csp_YGPGuard — ygdy8.com scraper
    └── wogg.rs       # csp_WoGGGuard — Cloud-drive based spider

crates/rivu-ui/src/
└── app.rs            # Replace `api: SiteApi` with `engine: SpiderApi`
```

### Spider Trait

Spider methods receive `&Site` (not just `ext`) so they can read `site.ext`,
`site.key`, `site.api`, etc. — each Spider uses the fields it needs.

```rust
#[async_trait]
pub trait Spider: Send + Sync {
    fn name(&self) -> &str;
    async fn home(&self, site: &Site) -> Result<ApiResult>;
    async fn category(&self, site: &Site, tid: &str, pg: i32, filters: &[(&str, &str)]) -> Result<ApiResult>;
    async fn detail(&self, site: &Site, ids: &[String]) -> Result<ApiResult>;
    async fn play(&self, site: &Site, flag: &str, id: &str) -> Result<PlayInfo>;
    async fn search(&self, site: &Site, keyword: &str, pg: i32) -> Result<ApiResult>;
}
```

### SpiderRegistry

```rust
pub struct SpiderRegistry {
    spiders: HashMap<String, Box<dyn Spider>>,
}
```

- Registered at startup by Spider module (`register_builtin()`)
- `get(csp_name) -> Option<&dyn Spider>` lookup
- `names() -> Vec<String>` for debugging

### SpiderApi (Unified API Router)

A struct wrapping both SiteApi and SpiderRegistry, dispatching by site type.

```rust
pub struct SpiderApi {
    http: SiteApi,
    registry: SpiderRegistry,
}
```

Methods:
- `home(&self, site: &Site) -> Result<ApiResult>`
  - `site.site_type == 3` → lookup `SpiderRegistry` by `site.api` (`"csp_BiliGuard"`), call `spider.home(site)`
  - else → `self.http.home(site)`
- `category(...)`, `detail(...)`, `play(...)`, `search(...)` — same dispatch pattern

### App Integration

```rust
// Before
pub struct App {
    pub api: SiteApi,
}

// After
pub struct App {
    pub engine: SpiderApi,   // handles both HTTP API and Spider routing
}

// SpiderApi creation:
let mut registry = SpiderRegistry::new();
registry.register(Box::new(BiliSpider));
registry.register(Box::new(YGPSpider));
registry.register(Box::new(WoGGSpider));
let engine = SpiderApi::new(SiteApi::new(), registry);

// load_home (no site_type check needed):
fn load_home(&mut self) {
    let site = ...;
    let result = RT.block_on(self.engine.home(&site));
    // ... same result handling ...
}
```

All `if site_type == 3` checks in `app.rs` are removed — routing happens inside `SpiderApi::home()`.

`SpiderApi::new()` also pre-inserts `SiteApi` as a fallback for non-type-3 sites. The caller (main.rs or App::new) creates `SpiderApi` once and injects it into App.

### Data Flow

```
Enter → load_home()
  → RT.block_on(engine.home(&site))
    → site.type == 3
      → registry.get("csp_BiliGuard")
        → BiliSpider.home(&site)  # Spider reads site.ext, site.key etc.
          → HTTP GET to Bilibili API
          → Parse JSON → ApiResult
    → site.type == 0/1/2/4
      → SiteApi.home(&site)
        → Existing HTTP API logic
```

## Spider Implementations

### 1. BiliSpider (csp_BiliGuard)

Bilibili has open, well-documented public APIs. No authentication needed for
read-only access.

**Endpoints:**

| Operation | URL | Notes |
|-----------|-----|-------|
| home  | `https://api.bilibili.com/x/web-interface/popular` | Popular videos → site categories + list |
| category | `https://api.bilibili.com/x/web-interface/newlist?rid={tid}&pn={pg}` | Region + page |
| detail | `https://api.bilibili.com/x/web-interface/view?aid={id}` | Video metadata |
| play | `https://api.bilibili.com/x/player/playurl?avid={id}&cid={cid}&qn=80` | Requires `aid`+`cid`; detail provides `cid` |
| search | `https://api.bilibili.com/x/web-interface/search/all/v2?keyword={kw}&page={pg}` | Full-text search |

**Category mapping:** TVBox `type_id` → Bilibili `rid`:
- 1 → 1 (动画), 2 → 3 (音乐), 3 → 4 (游戏), 4 → 5 (知识),
- 5 → 11 (影视), 6 → 21 (纪录片), 7 → 23 (电影), 8 → 24 (电视剧)

**Response mapping:**
- Bilibili JSON → TVBox `ApiResult` with `class` and `list`
- Bilibili video → TVBox `Vod` (vod_id = aid, vod_pic = pic, vod_name = title, etc.)
- Bilibili play URL → TVBox `PlayInfo` (url, referer = `https://www.bilibili.com`)

**Ext:** Not used (Bilibili API is self-contained).

### 2. YGPSpider (csp_YGPGuard)

阳光电影 (ygdy8.com) — HTML scraping based.

**Dependency:** `scraper = "0.20"` added to `rivu-spider/Cargo.toml`.

**Endpoints:**

| Operation | URL | Notes |
|-----------|-----|-------|
| home | `https://www.ygdy8.com/html/gndy/dyzz/index.html` | Recent movies |
| category | `https://www.ygdy8.com/html/gndy/dyzz/list_{tid}_{pg}.html` | tid=1 (dyzz), 2 (oumei), … |
| detail | `https://www.ygdy8.com{tom}` | From list page href |
| play | Extract magnet/ed2k from detail page | Direct link |
| search | `https://s.ygdy8.com/plus/s0.php?typeid=1&keyword={kw}` | Search |

**Scraping approach:** Use `scraper` crate `Html::parse_document` + CSS selectors.
- List pages: `table.tbspan a.ulink` for movie links
- Detail page: `div#Zoom span` for content, extract `thunder://` or `magnet:?` links
- Home categories: hardcoded map of `tid` → category name

**Ext:** Not used initially.

### 3. WoGGSpider (csp_WoGGGuard)

Cloud-drive based Spider. Reads a remote text file listing cloud drive content.

**Ext format:**
```json
{"Cloud-drive": "tvfan/Cloud-drive.txt"}
```

**Behavior:**
1. The Spider uses the source URL's base as the CDN root.
   Source URL `http://www.饭太硬.cc/tv` → base `http://www.饭太硬.cc/`.
   The Spider receives the full `Site` including its `ext` field containing
   `{"Cloud-drive": "tvfan/Cloud-drive.txt"}`.
2. Fetch `http://www.饭太硬.cc/tvfan/Cloud-drive.txt` (base + ext path).
3. Parse line-delimited entries: `name$$url$$type` (drive name, URL, media type).
4. Each line becomes a Vod with play URL pointing to the cloud drive.
5. Categories are derived from type field (video/audio/etc.).

**Ext:** Required — contains the cloud drive list path.

## Error Handling

- Unknown `csp_*` name → `CoreError::Spider("Spider 'csp_XxxGuard' not implemented")`
- HTTP failure in Spider → `CoreError::Http(...)` (wrapped)
- Parse failure → `CoreError::Json(...)` (wrapped)
- Empty results → `ApiResult { class: None, list: None }` (not an error)

## Testing Strategy

### Unit Tests (per Spider)
- `test_bili_home_returns_api_result` — mock HTTP, verify mapping
- `test_bili_category_parses_response`
- `test_bili_detail_extracts_cid`
- `test_bili_play_builds_url`
- Similar for YGP and WoGG

### Registry Tests
- `test_registry_lookup_found`
- `test_registry_lookup_not_found_returns_error`
- `test_registry_builtin_contains_bili_ygp_wogg`

### Integration Tests
- `test_spider_api_routes_type_3_to_spider`
- `test_spider_api_routes_type_0_to_site_api`
- `test_app_load_home_with_spider_site`
- `test_spider_api_unknown_csp_returns_clear_error`

## Non-Goals (Future Work)

- JS/drpy Spider runtime — can be added as a Spider implementation later
- JAR loading — rarely used in modern sources
- Dynamic Spider loading from remote configs
- All 48 饭太硬 spiders — start with 3 most popular, add on demand
