use serde::Deserialize;

/// A post's content language.
///
/// The blog is Thai-first, so [`Lang::Th`] is the default and an English post
/// must opt in with `lang: en` in its frontmatter. The value drives the `lang`
/// attribute of the rendered article and the small language badge shown for
/// posts that differ from the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
  #[default]
  Th,
  En,
}

impl Lang {
  /// BCP 47 language tag (`th` / `en`), for the `lang` attribute.
  pub fn code(self) -> &'static str {
    match self {
      Lang::Th => "th",
      Lang::En => "en",
    }
  }

  /// Short badge label (`TH` / `EN`).
  pub fn badge(self) -> &'static str {
    match self {
      Lang::Th => "TH",
      Lang::En => "EN",
    }
  }

  /// Full language name, for `aria-label` and tooltips.
  pub fn label(self) -> &'static str {
    match self {
      Lang::Th => "ภาษาไทย",
      Lang::En => "English",
    }
  }
}

/// Post metadata shared by the RSS build (`build.rs`) and the in-app render
/// (`content.rs`). Both call [`parse`] so the feed and the page can never
/// disagree about a post's title, date, slug, tags, or language.
#[derive(Debug, Clone, PartialEq)]
pub struct Frontmatter {
  pub title: String,
  /// Validated `YYYY-MM-DD` calendar date (see [`parse`]).
  pub date: String,
  pub description: String,
  pub tags: Vec<String>,
  pub author: Option<String>,
  /// Content language; defaults to Thai when the frontmatter omits `lang`.
  pub lang: Lang,
  pub draft: bool,
  pub slug: Option<Slug>,
}

/// A successfully parsed post: frontmatter plus the markdown body with a
/// single, shared leading-newline convention.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
  pub meta: Frontmatter,
  pub body: String,
  /// Non-fatal observations (e.g. unknown frontmatter keys), sorted for
  /// determinism. Callers decide whether to warn (build.rs) or ignore.
  pub warnings: Vec<String>,
}

/// Why a post file failed to parse. Errors are typed so callers can decide
/// whether to skip silently (runtime) or fail the build loudly (build.rs).
#[derive(Debug, Clone, PartialEq)]
pub enum FrontmatterError {
  /// File does not start with a `---` frontmatter delimiter.
  MissingDelimiter,
  /// Opening `---` exists but no closing `---` line was found.
  MissingClosingDelimiter,
  /// Frontmatter is not valid YAML / does not match the schema.
  InvalidYaml(String),
  /// The required `date` field is absent.
  MissingDate,
  /// `date` is not a valid `YYYY-MM-DD` calendar date.
  InvalidDate(String),
  /// The `slug` field (or file stem) is not a safe URL path segment.
  InvalidSlug(String, SlugError),
}

impl std::fmt::Display for FrontmatterError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FrontmatterError::MissingDelimiter => {
        write!(f, "missing opening `---` frontmatter delimiter")
      }
      FrontmatterError::MissingClosingDelimiter => {
        write!(f, "missing closing `---` frontmatter delimiter")
      }
      FrontmatterError::InvalidYaml(msg) => write!(f, "invalid frontmatter: {msg}"),
      FrontmatterError::MissingDate => {
        write!(f, "frontmatter is missing the required `date` field")
      }
      FrontmatterError::InvalidDate(date) => {
        write!(
          f,
          "`date` is not a valid YYYY-MM-DD calendar date: {date:?}"
        )
      }
      FrontmatterError::InvalidSlug(slug, reason) => {
        write!(f, "`slug` is not a safe URL slug ({reason}): {slug:?}")
      }
    }
  }
}

impl std::error::Error for FrontmatterError {}

/// A validated post slug.
///
/// A slug becomes both a URL path segment (`/post/<slug>`) and a directory
/// name (`dist/post/<slug>/`), so only non-empty ASCII lowercase letters,
/// digits, and hyphens are representable. Path separators, `.` (and thus
/// `..`), whitespace, and the XML-special characters `&`, `<`, `>` cannot be
/// constructed at all - path traversal and unescaped sitemap output are
/// prevented by the type, not merely checked at each call site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Slug(String);

/// Why a string cannot become a [`Slug`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlugError {
  /// Slugs must not be empty.
  Empty,
  /// Slugs must contain only `a-z`, `0-9`, and `-`.
  InvalidCharacter(char),
}

impl Slug {
  /// The slug as a string slice.
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Validate `raw` as a slug.
  pub fn new(raw: &str) -> Result<Self, SlugError> {
    if raw.is_empty() {
      return Err(SlugError::Empty);
    }
    match raw
      .chars()
      .find(|c| !matches!(c, 'a'..='z' | '0'..='9' | '-'))
    {
      Some(c) => Err(SlugError::InvalidCharacter(c)),
      None => Ok(Self(raw.to_string())),
    }
  }
}

impl std::fmt::Display for SlugError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SlugError::Empty => write!(f, "must not be empty"),
      SlugError::InvalidCharacter(c) => {
        write!(
          f,
          "must contain only lowercase ASCII letters, digits, and hyphens, found {c:?}"
        )
      }
    }
  }
}

impl std::error::Error for SlugError {}

impl std::fmt::Display for Slug {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl std::str::FromStr for Slug {
  type Err = SlugError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::new(s)
  }
}

impl AsRef<str> for Slug {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl std::ops::Deref for Slug {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl PartialEq<str> for Slug {
  fn eq(&self, other: &str) -> bool {
    self.0 == other
  }
}

impl PartialEq<&str> for Slug {
  fn eq(&self, other: &&str) -> bool {
    self.0 == *other
  }
}

/// Lenient YAML shape so the parser can distinguish "absent" from "invalid".
#[derive(Debug, Deserialize)]
struct Raw {
  title: String,
  #[serde(default)]
  date: Option<String>,
  #[serde(default)]
  description: String,
  #[serde(default)]
  tags: Vec<String>,
  #[serde(default)]
  author: Option<String>,
  #[serde(default)]
  lang: Option<String>,
  #[serde(default)]
  draft: bool,
  #[serde(default)]
  slug: Option<String>,
}

/// Split a post file into frontmatter and body, validating both.
///
/// Tolerates a UTF-8 BOM and CRLF line endings. The body keeps exactly one
/// leading newline stripped, so `build.rs` and `content.rs` render the same
/// markdown.
pub fn parse(raw: &str) -> Result<Parsed, FrontmatterError> {
  let trimmed = raw.strip_prefix('\u{feff}').unwrap_or(raw);
  if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
    return Err(FrontmatterError::MissingDelimiter);
  }

  let rest = &trimmed[3..];
  let end = rest
    .find("\n---")
    .ok_or(FrontmatterError::MissingClosingDelimiter)?;
  let fm_text = &rest[..end];

  let mut warnings = Vec::new();
  let value: serde_yaml::Value =
    serde_yaml::from_str(fm_text).map_err(|e| FrontmatterError::InvalidYaml(e.to_string()))?;
  if let serde_yaml::Value::Mapping(map) = &value {
    for key in map.keys() {
      if let serde_yaml::Value::String(k) = key
        && !KNOWN_KEYS.contains(&k.as_str())
      {
        warnings.push(format!("unknown frontmatter key `{k}`"));
      }
    }
  }
  let raw_meta: Raw =
    serde_yaml::from_value(value).map_err(|e| FrontmatterError::InvalidYaml(e.to_string()))?;

  let date = raw_meta.date.ok_or(FrontmatterError::MissingDate)?;
  if !is_valid_iso_date(&date) {
    return Err(FrontmatterError::InvalidDate(date));
  }

  let lang = parse_lang(raw_meta.lang.as_deref(), &mut warnings);
  warnings.sort();

  let mut body = rest[end + 4..].to_string();
  if let Some(stripped) = body.strip_prefix('\n') {
    body = stripped.to_string();
  }

  let mut tags = raw_meta.tags;
  let mut seen = std::collections::HashSet::new();
  tags.retain(|t| seen.insert(t.clone()));

  let slug = raw_meta
    .slug
    .map(|raw| Slug::new(&raw).map_err(|reason| FrontmatterError::InvalidSlug(raw, reason)))
    .transpose()?;

  Ok(Parsed {
    meta: Frontmatter {
      title: raw_meta.title,
      date,
      description: raw_meta.description,
      tags,
      author: raw_meta.author,
      lang,
      draft: raw_meta.draft,
      slug,
    },
    body,
    warnings,
  })
}

/// Map the raw `lang` string to a [`Lang`], warning on unknown values so a
/// typo surfaces instead of silently falling back.
fn parse_lang(raw: Option<&str>, warnings: &mut Vec<String>) -> Lang {
  match raw {
    None | Some("th") => Lang::Th,
    Some("en") => Lang::En,
    Some(other) => {
      warnings.push(format!(
        "unknown `lang` value `{other}` (expected `th` or `en`), defaulting to `th`"
      ));
      Lang::Th
    }
  }
}

/// Keys the frontmatter schema accepts; anything else is reported as a
/// warning so typos surface instead of being silently dropped.
const KNOWN_KEYS: [&str; 8] = [
  "title",
  "date",
  "description",
  "tags",
  "author",
  "lang",
  "draft",
  "slug",
];

/// Strict `YYYY-MM-DD` check: exact digit counts, plus a real calendar check
/// (leap years included). No chrono dependency in the WASM binary.
fn is_valid_iso_date(s: &str) -> bool {
  let mut parts = s.split('-');
  let (Some(year), Some(month), Some(day), None) =
    (parts.next(), parts.next(), parts.next(), parts.next())
  else {
    return false;
  };
  let is_digits = |p: &str| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit());
  if year.len() != 4
    || month.len() != 2
    || day.len() != 2
    || !is_digits(year)
    || !is_digits(month)
    || !is_digits(day)
  {
    return false;
  }
  let (year, month, day): (u32, u32, u32) = match (year.parse(), month.parse(), day.parse()) {
    (Ok(y), Ok(m), Ok(d)) => (y, m, d),
    _ => return false,
  };
  if !(1..=12).contains(&month) || day == 0 {
    return false;
  }
  let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
  let days_in_month = [
    31,
    if leap { 29 } else { 28 },
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ];
  day <= days_in_month[(month - 1) as usize]
}

/// A post's URL slug: the frontmatter `slug` override (already validated by
/// [`parse`]), else the validated file stem. Shared by `build.rs`
/// (RSS/sitemap) and `content.rs` (routing, static pages), so the feed, the
/// router, and `dist/post/<slug>/` can never disagree.
pub fn derive_slug(meta: &Frontmatter, file_stem: Option<&str>) -> Result<Slug, FrontmatterError> {
  if let Some(slug) = &meta.slug {
    return Ok(slug.clone());
  }
  let stem = file_stem.unwrap_or_default();
  Slug::new(stem).map_err(|reason| FrontmatterError::InvalidSlug(stem.to_string(), reason))
}

/// Advisory checks beyond the hard parse errors, in the same order every
/// time. Drafts are work-in-progress by contract, so they are not linted
/// (a future date or missing description is expected mid-writing).
///
/// `today_iso` is an ISO `YYYY-MM-DD` string; ISO dates compare correctly
/// as strings, so no calendar dependency is needed in the WASM binary.
/// `tag_counts` maps every tag to how many published posts use it.
pub fn lint_post(
  meta: &Frontmatter,
  today_iso: &str,
  tag_counts: &std::collections::HashMap<String, usize>,
) -> Vec<String> {
  if meta.draft {
    return Vec::new();
  }
  let mut out = Vec::new();
  if meta.description.is_empty() {
    out.push("missing `description` (used by post cards, RSS, and OG meta)".to_string());
  }
  if meta.date.as_str() > today_iso {
    out.push(format!(
      "`date` ({}) is in the future - set `draft: true` until it is published",
      meta.date
    ));
  }
  for tag in &meta.tags {
    if tag_counts.get(tag).copied().unwrap_or(0) <= 1 {
      out.push(format!(
        "tag `{tag}` is used by no other post - possible typo?"
      ));
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample() -> String {
    "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\ntags: [rust]\n---\nBody text".to_string()
  }

  #[test]
  fn parses_valid_post() {
    let parsed = parse(&sample()).unwrap();
    assert_eq!(parsed.meta.title, "Hello");
    assert_eq!(parsed.meta.date, "2026-08-29");
    assert_eq!(parsed.meta.tags, vec!["rust"]);
    assert_eq!(parsed.body, "Body text");
  }

  #[test]
  fn blank_line_after_delimiter_keeps_one_leading_newline() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\n---\n\nBody text";
    assert_eq!(parse(raw).unwrap().body, "\nBody text");
  }

  #[test]
  fn tolerates_bom() {
    let mut raw = "\u{feff}".to_string();
    raw.push_str(&sample());
    assert!(parse(&raw).is_ok());
  }

  #[test]
  fn tolerates_crlf() {
    let raw = "---\r\ntitle: \"Hello\"\r\ndate: \"2026-08-29\"\r\n---\r\nBody text";
    let parsed = parse(raw).unwrap();
    // CRLF files keep the \r\n separator before the body, exactly like the
    // parser they replaced - behavior parity, not a fix-up.
    assert_eq!(parsed.body, "\r\nBody text");
  }

  #[test]
  fn missing_opening_delimiter_is_an_error() {
    assert_eq!(
      parse("title: \"Hello\"\n"),
      Err(FrontmatterError::MissingDelimiter)
    );
  }

  #[test]
  fn missing_closing_delimiter_is_an_error() {
    assert!(matches!(
      parse("---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\n"),
      Err(FrontmatterError::MissingClosingDelimiter)
    ));
  }

  #[test]
  fn missing_date_is_an_error() {
    let raw = "---\ntitle: \"Hello\"\n---\nBody";
    assert_eq!(parse(raw), Err(FrontmatterError::MissingDate));
  }

  #[test]
  fn rejects_invalid_calendar_dates() {
    for bad in [
      "2024-02-30",
      "2024-13-01",
      "2023-02-29",
      "",
      "2024-1-1",
      "not-a-date",
      "26-08-29",
    ] {
      let raw = format!("---\ntitle: \"Hello\"\ndate: \"{bad}\"\n---\nBody");
      assert_eq!(
        parse(&raw),
        Err(FrontmatterError::InvalidDate(bad.to_string())),
        "date {bad:?}"
      );
    }
  }

  #[test]
  fn accepts_valid_calendar_dates() {
    for good in ["2024-02-29", "2026-08-29", "1900-01-01", "2000-12-31"] {
      let raw = format!("---\ntitle: \"Hello\"\ndate: \"{good}\"\n---\nBody");
      assert!(parse(&raw).is_ok(), "date {good:?} should be valid");
    }
  }

  #[test]
  fn slug_override_survives_parse() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\nslug: \"custom-slug\"\n---\nBody";
    assert_eq!(
      parse(raw).unwrap().meta.slug.as_deref(),
      Some("custom-slug")
    );
  }

  #[test]
  fn rejects_unsafe_slugs() {
    for bad in [
      "",
      "/tmp/rust-blog-poc",
      "../../x",
      "a/b",
      "..",
      "Upper",
      "with space",
      "under_score",
      "a.b",
      "a&b",
      "a<b",
      "ภาษาไทย",
    ] {
      let raw = format!("---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\nslug: \"{bad}\"\n---\nBody");
      assert!(
        matches!(parse(&raw), Err(FrontmatterError::InvalidSlug(ref s, _)) if s == bad),
        "slug {bad:?} must be rejected"
      );
    }
  }

  #[test]
  fn accepts_safe_slugs() {
    for good in ["welcome", "cargo-deny", "post-2", "123", "a"] {
      let raw = format!("---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\nslug: \"{good}\"\n---\nBody");
      let slug = parse(&raw).unwrap().meta.slug.expect("slug survives parse");
      assert_eq!(slug, good);
    }
  }

  #[test]
  fn slug_from_str_uses_the_same_rule() {
    assert_eq!(
      "cargo-deny".parse::<Slug>().unwrap(),
      Slug::new("cargo-deny").unwrap()
    );
    assert!("Cargo".parse::<Slug>().is_err());
  }

  #[test]
  fn derive_slug_validates_the_file_stem() {
    let no_override = meta("2026-08-29", &[], false);
    assert_eq!(
      derive_slug(&no_override, Some("Bad_Name")).unwrap_err(),
      FrontmatterError::InvalidSlug("Bad_Name".to_string(), SlugError::InvalidCharacter('B'))
    );
    assert_eq!(
      derive_slug(&no_override, Some("good-name")).unwrap(),
      Slug::new("good-name").unwrap()
    );
    assert_eq!(
      derive_slug(&no_override, None).unwrap_err(),
      FrontmatterError::InvalidSlug(String::new(), SlugError::Empty)
    );
  }

  #[test]
  fn derive_slug_prefers_the_validated_override() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\nslug: \"chosen\"\n---\nBody";
    let parsed = parse(raw).unwrap();
    // The override wins even when the file stem would fail validation.
    assert_eq!(
      derive_slug(&parsed.meta, Some("Bad Stem!")).unwrap(),
      Slug::new("chosen").unwrap()
    );
  }

  #[test]
  fn invalid_yaml_is_a_typed_error() {
    let raw = "---\ntitle: [unclosed\n---\nBody";
    assert!(matches!(parse(raw), Err(FrontmatterError::InvalidYaml(_))));
  }

  #[test]
  fn missing_title_is_a_typed_error() {
    let raw = "---\ndate: \"2026-08-29\"\n---\nBody";
    assert!(matches!(
      parse(raw),
      Err(FrontmatterError::InvalidYaml(msg)) if msg.contains("title")
    ));
  }

  #[test]
  fn unknown_keys_are_reported_as_warnings() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\ncatagories: [oops]\n---\nBody";
    let parsed = parse(raw).unwrap();
    assert_eq!(
      parsed.warnings,
      vec!["unknown frontmatter key `catagories`"]
    );
    assert!(parse(&sample()).unwrap().warnings.is_empty());
  }

  #[test]
  fn lang_defaults_to_thai_when_absent() {
    assert_eq!(parse(&sample()).unwrap().meta.lang, Lang::Th);
  }

  #[test]
  fn lang_parses_english() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\nlang: en\n---\nBody";
    assert_eq!(parse(raw).unwrap().meta.lang, Lang::En);
  }

  #[test]
  fn lang_parses_explicit_thai() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\nlang: th\n---\nBody";
    assert_eq!(parse(raw).unwrap().meta.lang, Lang::Th);
  }

  #[test]
  fn unknown_lang_warns_and_defaults_to_thai() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\nlang: fr\n---\nBody";
    let parsed = parse(raw).unwrap();
    assert_eq!(parsed.meta.lang, Lang::Th);
    assert_eq!(
      parsed.warnings,
      vec!["unknown `lang` value `fr` (expected `th` or `en`), defaulting to `th`"]
    );
  }

  #[test]
  fn lang_maps_to_code_badge_and_label() {
    assert_eq!(Lang::Th.code(), "th");
    assert_eq!(Lang::En.code(), "en");
    assert_eq!(Lang::Th.badge(), "TH");
    assert_eq!(Lang::En.badge(), "EN");
    assert_eq!(Lang::Th.label(), "ภาษาไทย");
    assert_eq!(Lang::En.label(), "English");
  }

  #[test]
  fn duplicate_tags_are_deduplicated() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\ntags: [rust, wasm, rust]\n---\nBody";
    assert_eq!(parse(raw).unwrap().meta.tags, vec!["rust", "wasm"]);
  }

  fn counts(entries: &[(&str, usize)]) -> std::collections::HashMap<String, usize> {
    entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
  }

  fn meta(date: &str, tags: &[&str], draft: bool) -> Frontmatter {
    Frontmatter {
      title: "Hello".to_string(),
      date: date.to_string(),
      description: "A description".to_string(),
      tags: tags.iter().map(|t| t.to_string()).collect(),
      author: None,
      lang: Lang::Th,
      draft,
      slug: None,
    }
  }

  #[test]
  fn lint_warns_on_missing_description_and_future_date() {
    let today = "2026-08-29";
    let mut post = meta("2027-01-01", &["rust"], false);
    post.description = String::new();
    let warnings = lint_post(&post, today, &counts(&[("rust", 2)]));
    assert_eq!(
      warnings,
      vec![
        "missing `description` (used by post cards, RSS, and OG meta)".to_string(),
        "`date` (2027-01-01) is in the future - set `draft: true` until it is published"
          .to_string()
      ]
    );
  }

  #[test]
  fn lint_warns_on_single_use_tag() {
    let warnings = lint_post(
      &meta("2026-08-29", &["ruts"], false),
      "2026-08-29",
      &counts(&[("ruts", 1)]),
    );
    assert_eq!(
      warnings,
      vec!["tag `ruts` is used by no other post - possible typo?"]
    );
  }

  #[test]
  fn lint_skips_drafts() {
    let warnings = lint_post(
      &meta("2027-01-01", &["ruts"], true),
      "2026-08-29",
      &counts(&[("ruts", 1)]),
    );
    assert!(
      warnings.is_empty(),
      "drafts are work-in-progress: {warnings:?}"
    );
  }

  #[test]
  fn lint_is_quiet_for_clean_posts() {
    let warnings = lint_post(
      &meta("2026-08-29", &["rust", "wasm"], false),
      "2026-08-29",
      &counts(&[("rust", 2), ("wasm", 2)]),
    );
    assert!(
      warnings.is_empty(),
      "clean post must lint clean: {warnings:?}"
    );
  }
}
