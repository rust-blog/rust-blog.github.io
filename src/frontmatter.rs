use serde::Deserialize;

/// Post metadata shared by the RSS build (`build.rs`) and the in-app render
/// (`content.rs`). Both call [`parse`] so the feed and the page can never
/// disagree about a post's title, date, slug, or tags.
#[derive(Debug, Clone, PartialEq)]
pub struct Frontmatter {
  pub title: String,
  /// Validated `YYYY-MM-DD` calendar date (see [`parse`]).
  pub date: String,
  pub description: String,
  pub tags: Vec<String>,
  pub author: Option<String>,
  pub draft: bool,
  pub slug: Option<String>,
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
    }
  }
}

impl std::error::Error for FrontmatterError {}

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
  warnings.sort();
  let raw_meta: Raw =
    serde_yaml::from_value(value).map_err(|e| FrontmatterError::InvalidYaml(e.to_string()))?;

  let date = raw_meta.date.ok_or(FrontmatterError::MissingDate)?;
  if !is_valid_iso_date(&date) {
    return Err(FrontmatterError::InvalidDate(date));
  }

  let mut body = rest[end + 4..].to_string();
  if let Some(stripped) = body.strip_prefix('\n') {
    body = stripped.to_string();
  }

  let mut tags = raw_meta.tags;
  let mut seen = std::collections::HashSet::new();
  tags.retain(|t| seen.insert(t.clone()));

  Ok(Parsed {
    meta: Frontmatter {
      title: raw_meta.title,
      date,
      description: raw_meta.description,
      tags,
      author: raw_meta.author,
      draft: raw_meta.draft,
      slug: raw_meta.slug,
    },
    body,
    warnings,
  })
}

/// Keys the frontmatter schema accepts; anything else is reported as a
/// warning so typos surface instead of being silently dropped.
const KNOWN_KEYS: [&str; 7] = [
  "title",
  "date",
  "description",
  "tags",
  "author",
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
    assert_eq!(parse(&raw).unwrap().body, "\nBody text");
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
      parse(&raw).unwrap().meta.slug.as_deref(),
      Some("custom-slug")
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
  fn duplicate_tags_are_deduplicated() {
    let raw = "---\ntitle: \"Hello\"\ndate: \"2026-08-29\"\ntags: [rust, wasm, rust]\n---\nBody";
    assert_eq!(parse(raw).unwrap().meta.tags, vec!["rust", "wasm"]);
  }
}
