use std::borrow::Cow;

use polars_arrow::export::arrow::array::{MutableArray, MutableUtf8Array};
use polars_arrow::utils::CustomIterTools;
#[cfg(feature = "regex")]
use regex::{escape, Regex};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum StringFunction {
    Contains {
        pat: String,
        literal: bool,
    },
    StartsWith(String),
    EndsWith(String),
    Extract {
        pat: String,
        group_index: usize,
    },
    #[cfg(feature = "string_justify")]
    Zfill(usize),
    #[cfg(feature = "string_justify")]
    LJust {
        width: usize,
        fillchar: char,
    },
    #[cfg(feature = "string_justify")]
    RJust {
        width: usize,
        fillchar: char,
    },
    ExtractAll,
    CountMatch(String),
    #[cfg(feature = "temporal")]
    Strptime(StrpTimeOptions),
    #[cfg(feature = "concat_str")]
    ConcatVertical(String),
    #[cfg(feature = "concat_str")]
    ConcatHorizontal(String),
    #[cfg(feature = "regex")]
    Replace {
        // replace_single or replace_all
        all: bool,
        literal: bool,
    },
    Uppercase,
    Lowercase,
    Strip(Option<String>),
    RStrip(Option<String>),
    LStrip(Option<String>),
    Split {
        by: String,
        inclusive: bool,
    },
    SplitExact {
        by: String,
        inclusive: bool,
        n: usize,
    },
    SplitN {
        by: String,
        n: usize,
    },
}

impl Display for StringFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use StringFunction::*;
        let s = match self {
            Contains { .. } => "contains",
            StartsWith(_) => "starts_with",
            EndsWith(_) => "ends_with",
            Extract { .. } => "extract",
            #[cfg(feature = "string_justify")]
            Zfill(_) => "zfill",
            #[cfg(feature = "string_justify")]
            LJust { .. } => "str.ljust",
            #[cfg(feature = "string_justify")]
            RJust { .. } => "rjust",
            ExtractAll => "extract_all",
            CountMatch(_) => "count_match",
            #[cfg(feature = "temporal")]
            Strptime(_) => "strptime",
            #[cfg(feature = "concat_str")]
            ConcatVertical(_) => "concat_vertical",
            #[cfg(feature = "concat_str")]
            ConcatHorizontal(_) => "concat_horizontal",
            #[cfg(feature = "regex")]
            Replace { .. } => "replace",
            Uppercase => "uppercase",
            Lowercase => "lowercase",
            Strip(_) => "strip",
            LStrip(_) => "lstrip",
            RStrip(_) => "rstrip",
            Split { inclusive, .. } => {
                if *inclusive {
                    "split_inclusive"
                } else {
                    "split"
                }
            }
            SplitExact { inclusive, .. } => {
                if *inclusive {
                    "split_exact_inclusive"
                } else {
                    "split_exact"
                }
            }
            SplitN { by, n } => "splitn",
        };

        write!(f, "str.{s}")
    }
}

pub(super) fn uppercase(s: &Series) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    Ok(ca.to_uppercase().into_series())
}

pub(super) fn lowercase(s: &Series) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    Ok(ca.to_lowercase().into_series())
}

pub(super) fn contains(s: &Series, pat: &str, literal: bool) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    if literal {
        ca.contains_literal(pat).map(|ca| ca.into_series())
    } else {
        ca.contains(pat).map(|ca| ca.into_series())
    }
}

pub(super) fn ends_with(s: &Series, sub: &str) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    Ok(ca.ends_with(sub).into_series())
}
pub(super) fn starts_with(s: &Series, sub: &str) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    Ok(ca.starts_with(sub).into_series())
}

/// Extract a regex pattern from the a string value.
pub(super) fn extract(s: &Series, pat: &str, group_index: usize) -> PolarsResult<Series> {
    let pat = pat.to_string();

    let ca = s.utf8()?;
    ca.extract(&pat, group_index).map(|ca| ca.into_series())
}

#[cfg(feature = "string_justify")]
pub(super) fn zfill(s: &Series, alignment: usize) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    Ok(ca.zfill(alignment).into_series())
}

#[cfg(feature = "string_justify")]
pub(super) fn ljust(s: &Series, width: usize, fillchar: char) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    Ok(ca.ljust(width, fillchar).into_series())
}
#[cfg(feature = "string_justify")]
pub(super) fn rjust(s: &Series, width: usize, fillchar: char) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    Ok(ca.rjust(width, fillchar).into_series())
}

pub(super) fn strip(s: &Series, matches: Option<&str>) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    if let Some(matches) = matches {
        if matches.chars().count() == 1 {
            // Fast path for when a single character is passed
            Ok(ca
                .apply(|s| Cow::Borrowed(s.trim_matches(matches.chars().next().unwrap())))
                .into_series())
        } else {
            Ok(ca
                .apply(|s| Cow::Borrowed(s.trim_matches(|c| matches.contains(c))))
                .into_series())
        }
    } else {
        Ok(ca.apply(|s| Cow::Borrowed(s.trim())).into_series())
    }
}

pub(super) fn lstrip(s: &Series, matches: Option<&str>) -> PolarsResult<Series> {
    let ca = s.utf8()?;

    if let Some(matches) = matches {
        if matches.chars().count() == 1 {
            // Fast path for when a single character is passed
            Ok(ca
                .apply(|s| Cow::Borrowed(s.trim_start_matches(matches.chars().next().unwrap())))
                .into_series())
        } else {
            Ok(ca
                .apply(|s| Cow::Borrowed(s.trim_start_matches(|c| matches.contains(c))))
                .into_series())
        }
    } else {
        Ok(ca.apply(|s| Cow::Borrowed(s.trim_start())).into_series())
    }
}

pub(super) fn rstrip(s: &Series, matches: Option<&str>) -> PolarsResult<Series> {
    let ca = s.utf8()?;
    if let Some(matches) = matches {
        if matches.chars().count() == 1 {
            // Fast path for when a single character is passed
            Ok(ca
                .apply(|s| Cow::Borrowed(s.trim_end_matches(matches.chars().next().unwrap())))
                .into_series())
        } else {
            Ok(ca
                .apply(|s| Cow::Borrowed(s.trim_end_matches(|c| matches.contains(c))))
                .into_series())
        }
    } else {
        Ok(ca.apply(|s| Cow::Borrowed(s.trim_end())).into_series())
    }
}

pub(super) fn extract_all(args: &[Series]) -> PolarsResult<Series> {
    let s = &args[0];
    let pat = &args[1];

    let ca = s.utf8()?;
    let pat = pat.utf8()?;

    if pat.len() == 1 {
        let pat = pat
            .get(0)
            .ok_or_else(|| PolarsError::ComputeError("Expected a pattern got null".into()))?;
        ca.extract_all(pat).map(|ca| ca.into_series())
    } else {
        ca.extract_all_many(pat).map(|ca| ca.into_series())
    }
}

pub(super) fn count_match(s: &Series, pat: &str) -> PolarsResult<Series> {
    let pat = pat.to_string();

    let ca = s.utf8()?;
    ca.count_match(&pat).map(|ca| ca.into_series())
}

#[cfg(feature = "temporal")]
pub(super) fn strptime(s: &Series, options: &StrpTimeOptions) -> PolarsResult<Series> {
    let ca = s.utf8()?;

    let out = match &options.date_dtype {
        DataType::Date => {
            if options.exact {
                ca.as_date(options.fmt.as_deref(), options.cache)?
                    .into_series()
            } else {
                ca.as_date_not_exact(options.fmt.as_deref())?.into_series()
            }
        }
        DataType::Datetime(tu, _) => {
            if options.exact {
                ca.as_datetime(options.fmt.as_deref(), *tu, options.cache, options.tz_aware)?
                    .into_series()
            } else {
                ca.as_datetime_not_exact(options.fmt.as_deref(), *tu)?
                    .into_series()
            }
        }
        DataType::Time => {
            if options.exact {
                ca.as_time(options.fmt.as_deref(), options.cache)?
                    .into_series()
            } else {
                return Err(PolarsError::ComputeError(
                    format!("non-exact not implemented for dtype {:?}", DataType::Time).into(),
                ));
            }
        }
        dt => {
            return Err(PolarsError::ComputeError(
                format!("not implemented for dtype {dt:?}").into(),
            ))
        }
    };
    if options.strict {
        if out.null_count() != ca.null_count() {
            Err(PolarsError::ComputeError(
                "strict conversion to dates failed, maybe set strict=False".into(),
            ))
        } else {
            Ok(out.into_series())
        }
    } else {
        Ok(out.into_series())
    }
}

#[cfg(feature = "concat_str")]
pub(super) fn concat(s: &Series, delimiter: &str) -> PolarsResult<Series> {
    Ok(s.str_concat(delimiter).into_series())
}

#[cfg(feature = "concat_str")]
pub(super) fn concat_hor(s: &[Series], delimiter: &str) -> PolarsResult<Series> {
    polars_core::functions::concat_str(s, delimiter).map(|ca| ca.into_series())
}

impl From<StringFunction> for FunctionExpr {
    fn from(str: StringFunction) -> Self {
        FunctionExpr::StringExpr(str)
    }
}

#[cfg(feature = "regex")]
fn get_pat(pat: &Utf8Chunked) -> PolarsResult<&str> {
    pat.get(0).ok_or_else(|| {
        PolarsError::ComputeError("pattern may not be 'null' in 'replace' expression".into())
    })
}

// used only if feature="regex"
#[allow(dead_code)]
fn iter_and_replace<'a, F>(ca: &'a Utf8Chunked, val: &'a Utf8Chunked, f: F) -> Utf8Chunked
where
    F: Fn(&'a str, &'a str) -> Cow<'a, str>,
{
    let mut out: Utf8Chunked = ca
        .into_iter()
        .zip(val.into_iter())
        .map(|(opt_src, opt_val)| match (opt_src, opt_val) {
            (Some(src), Some(val)) => Some(f(src, val)),
            _ => None,
        })
        .collect_trusted();

    out.rename(ca.name());
    out
}

#[cfg(feature = "regex")]
fn replace_single<'a>(
    ca: &'a Utf8Chunked,
    pat: &'a Utf8Chunked,
    val: &'a Utf8Chunked,
    literal: bool,
) -> PolarsResult<Utf8Chunked> {
    match (pat.len(), val.len()) {
        (1, 1) => {
            let pat = get_pat(pat)?;
            let val = val.get(0).ok_or_else(|| PolarsError::ComputeError("value may not be 'null' in 'replace' expression".into()))?;

            match literal {
                true => ca.replace_literal(pat, val),
                false => ca.replace(pat, val),
            }
        }
        (1, len_val) => {
            let mut pat = get_pat(pat)?.to_string();
            if len_val != ca.len() {
                return Err(PolarsError::ComputeError(format!("The replacement value expression in 'str.replace' should be equal to the length of the string column.\
                Got column length: {} and replacement value length: {}", ca.len(), len_val).into()))
            }

            if literal {
                pat = escape(&pat)
            }


            let reg = Regex::new(&pat)?;
            let lit = pat.chars().all(|c| !c.is_ascii_punctuation());

            let f = |s: &'a str, val: &'a str| {
                if lit && (s.len() <= 32) {
                    Cow::Owned(s.replacen(&pat, val, 1))
                } else {
                    reg.replace(s, val)
                }
            };
            Ok(iter_and_replace(ca, val, f))
        }
        _ => Err(PolarsError::ComputeError("A dynamic pattern length in the 'str.replace' expressions are not yet supported. Consider open a feature request for this.".into()))
    }
}

#[cfg(feature = "regex")]
fn replace_all<'a>(
    ca: &'a Utf8Chunked,
    pat: &'a Utf8Chunked,
    val: &'a Utf8Chunked,
    literal: bool,
) -> PolarsResult<Utf8Chunked> {
    match (pat.len(), val.len()) {
        (1, 1) => {
            let pat = get_pat(pat)?;
            let val = val.get(0).ok_or_else(|| PolarsError::ComputeError("value may not be 'null' in 'replace' expression".into()))?;

            match literal {
                true => ca.replace_literal_all(pat, val),
                false => ca.replace_all(pat, val),
            }
        }
        (1, len_val) => {
            let mut pat = get_pat(pat)?.to_string();
            if len_val != ca.len() {
                return Err(PolarsError::ComputeError(format!("The replacement value expression in 'str.replace' should be equal to the length of the string column.\
                Got column length: {} and replacement value length: {}", ca.len(), len_val).into()))
            }

            if literal {
                pat = escape(&pat)
            }

            let reg = Regex::new(&pat)?;

            let f = |s: &'a str, val: &'a str| {
                reg.replace_all(s, val)
            };
            Ok(iter_and_replace(ca, val, f))
        }
        _ => Err(PolarsError::ComputeError("A dynamic pattern length in the 'str.replace' expressions are not yet supported. Consider open a feature request for this.".into()))
    }
}

#[cfg(feature = "regex")]
pub(super) fn replace(s: &[Series], literal: bool, all: bool) -> PolarsResult<Series> {
    let column = &s[0];
    let pat = &s[1];
    let val = &s[2];

    let column = column.utf8()?;
    let pat = pat.utf8()?;
    let val = val.utf8()?;

    if all {
        replace_all(column, pat, val, literal)
    } else {
        replace_single(column, pat, val, literal)
    }
    .map(|ca| ca.into_series())
}

pub(super) fn split(s: &Series, by: &str, inclusive: bool) -> PolarsResult<Series> {
    let ca = s.utf8()?;

    let mut builder = ListUtf8ChunkedBuilder::new(s.name(), s.len(), ca.get_values_size());
    ca.into_iter().for_each(|opt_s| match opt_s {
        None => builder.append_null(),
        Some(s) => {
            if !inclusive {
                let iter = s.split(&by);
                builder.append_values_iter(iter);
            } else {
                let iter = s.split_inclusive(&by);
                builder.append_values_iter(iter);
            }
        }
    });
    Ok(builder.finish().into_series())
}

pub(super) fn split_exact(s: &Series, by: &str, inclusive: bool, n: usize) -> PolarsResult<Series> {
    let ca = s.utf8()?;

    let mut arrs = (0..n + 1)
        .map(|_| MutableUtf8Array::<i64>::with_capacity(ca.len()))
        .collect::<Vec<_>>();

    ca.into_iter().for_each(|opt_s| match opt_s {
        None => {
            for arr in &mut arrs {
                arr.push_null()
            }
        }
        Some(s) => {
            let mut arr_iter = arrs.iter_mut();
            if !inclusive {
                let split_iter = s.split(&by);
                (split_iter)
                    .zip(&mut arr_iter)
                    .for_each(|(splitted, arr)| arr.push(Some(splitted)));
            } else {
                let split_iter = s.split_inclusive(&by);
                (split_iter)
                    .zip(&mut arr_iter)
                    .for_each(|(splitted, arr)| arr.push(Some(splitted)));
            }
            // fill the remaining with null
            for arr in arr_iter {
                arr.push_null()
            }
        }
    });
    let fields = arrs
        .into_iter()
        .enumerate()
        .map(|(i, mut arr)| {
            Series::try_from((format!("field_{i}").as_str(), arr.as_box())).unwrap()
        })
        .collect::<Vec<_>>();
    Ok(StructChunked::new(ca.name(), &fields)?.into_series())
}

pub(super) fn splitn(s: &Series, by: &str, n: usize) -> PolarsResult<Series> {
    let ca = s.utf8()?;

    let mut arrs = (0..n)
        .map(|_| MutableUtf8Array::<i64>::with_capacity(ca.len()))
        .collect::<Vec<_>>();

    ca.into_iter().for_each(|opt_s| match opt_s {
        None => {
            for arr in &mut arrs {
                arr.push_null()
            }
        }
        Some(s) => {
            let mut arr_iter = arrs.iter_mut();
            let split_iter = s.splitn(n, &by);
            (split_iter)
                .zip(&mut arr_iter)
                .for_each(|(splitted, arr)| arr.push(Some(splitted)));
            // fill the remaining with null
            for arr in arr_iter {
                arr.push_null()
            }
        }
    });
    let fields = arrs
        .into_iter()
        .enumerate()
        .map(|(i, mut arr)| {
            Series::try_from((format!("field_{i}").as_str(), arr.as_box())).unwrap()
        })
        .collect::<Vec<_>>();
    Ok(StructChunked::new(ca.name(), &fields)?.into_series())
}
