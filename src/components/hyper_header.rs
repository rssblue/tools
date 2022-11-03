use serde::de::Error;
use serde::{Deserialize, Deserializer};

/// Copied from https://docs.rs/hyper/0.11.1/src/hyper/header/common/range.rs.html under the terms
/// of MIT license.
use std::fmt;
use std::str::FromStr;

/// `Range` header, defined in [RFC7233](https://tools.ietf.org/html/rfc7233#section-3.1)
///
/// The "Range" header field on a GET request modifies the method
/// semantics to request transfer of only one or more subranges of the
/// selected representation data, rather than the entire selected
/// representation data.
///
/// # ABNF
/// ```plain
/// Range =	byte-ranges-specifier / other-ranges-specifier
/// other-ranges-specifier = other-range-unit "=" other-range-set
/// other-range-set = 1*VCHAR
///
/// bytes-unit = "bytes"
///
/// byte-ranges-specifier = bytes-unit "=" byte-range-set
/// byte-range-set = 1#(byte-range-spec / suffix-byte-range-spec)
/// byte-range-spec = first-byte-pos "-" [last-byte-pos]
/// first-byte-pos = 1*DIGIT
/// last-byte-pos = 1*DIGIT
/// ```
///
/// # Example values
/// * `bytes=1000-`
/// * `bytes=-2000`
/// * `bytes=0-1,30-40`
/// * `bytes=0-10,20-90,-100`
/// * `custom_unit=0-123`
/// * `custom_unit=xxx-yyy`
///
/// # Examples
/// ```
/// use hyper::header::{Headers, Range, ByteRangeSpec};
///
/// let mut headers = Headers::new();
/// headers.set(Range::Bytes(
///     vec![ByteRangeSpec::FromTo(1, 100), ByteRangeSpec::AllFrom(200)]
/// ));
///
/// headers.clear();
/// headers.set(Range::Unregistered("letters".to_owned(), "a-f".to_owned()));
/// ```
/// ```
/// use hyper::header::{Headers, Range};
///
/// let mut headers = Headers::new();
/// headers.set(Range::bytes(1, 100));
///
/// headers.clear();
/// headers.set(Range::bytes_multi(vec![(1, 100), (200, 300)]));
/// ```
#[derive(PartialEq, Clone, Debug)]
pub enum Range {
    /// Byte range
    Bytes(Vec<ByteRangeSpec>),
    /// Custom range, with unit not registered at IANA
    /// (`other-range-unit`: String , `other-range-set`: String)
    Unregistered(String, String),
}

/// Each `Range::Bytes` header can contain one or more `ByteRangeSpecs`.
/// Each `ByteRangeSpec` defines a range of bytes to fetch
#[derive(PartialEq, Clone, Debug)]
pub enum ByteRangeSpec {
    /// Get all bytes between x and y ("x-y")
    FromTo(u64, u64),
    /// Get all bytes starting from x ("x-")
    AllFrom(u64),
    /// Get last x bytes ("-x")
    Last(u64),
}

impl fmt::Display for ByteRangeSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ByteRangeSpec::FromTo(from, to) => write!(f, "{}-{}", from, to),
            ByteRangeSpec::Last(pos) => write!(f, "-{}", pos),
            ByteRangeSpec::AllFrom(pos) => write!(f, "{}-", pos),
        }
    }
}

impl FromStr for Range {
    type Err = String;

    fn from_str(s: &str) -> Result<Range, Self::Err> {
        let mut iter = s.splitn(2, '=');

        match (iter.next(), iter.next()) {
            (Some("bytes"), Some(ranges)) => {
                let ranges = from_comma_delimited(ranges);
                if ranges.is_empty() {
                    return Err("No ranges found".to_owned());
                }
                Ok(Range::Bytes(ranges))
            }
            (Some(unit), Some(range_str)) if unit != "" && range_str != "" => {
                Ok(Range::Unregistered(unit.to_owned(), range_str.to_owned()))
            }
            _ => Err("Invalid Range header".to_owned()),
        }
    }
}

impl FromStr for ByteRangeSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<ByteRangeSpec, String> {
        let mut parts = s.splitn(2, '-');

        match (parts.next(), parts.next()) {
            (Some(""), Some(end)) => end
                .parse()
                .or(Err("Invalid last byte position".to_owned()))
                .map(ByteRangeSpec::Last),
            (Some(start), Some("")) => start
                .parse()
                .or(Err("Invalid first byte position".to_owned()))
                .map(ByteRangeSpec::AllFrom),
            (Some(start), Some(end)) => match (start.parse(), end.parse()) {
                (Ok(start), Ok(end)) if start <= end => Ok(ByteRangeSpec::FromTo(start, end)),
                _ => Err("Invalid byte range".to_owned()),
            },
            _ => Err("Invalid byte range".to_owned()),
        }
    }
}

fn from_comma_delimited<T: FromStr>(s: &str) -> Vec<T> {
    s.split(',')
        .filter_map(|x| match x.trim() {
            "" => None,
            y => Some(y),
        })
        .filter_map(|x| x.parse().ok())
        .collect()
}

impl<'de> Deserialize<'de> for Range {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = match String::deserialize(d) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        match Self::from_str(s.as_str()) {
            Ok(t) => Ok(t),
            Err(e) => Err(e).map_err(D::Error::custom),
        }
    }
}
