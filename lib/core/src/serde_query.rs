//! Deserializers for numbers that arrive as query-string text.
//!
//! `serde_urlencoded` normally deserializes `?page=2` into an `i32` fine: it
//! sees the field's type hint and parses the text itself. Adding
//! `#[serde(flatten)]` anywhere in the containing struct removes that hint —
//! serde buffers the input into its internal `Content` first, and every value
//! in the buffer is a *string*. Numeric fields **inside the flattened struct**
//! then fail with:
//!
//! ```text
//! invalid type: string "2", expected i32
//! ```
//!
//! Axum turns that rejection into `400 Bad Request` before the handler runs, so
//! the endpoint answers 400 to any request that names a page — while still
//! answering 200 when the client sends no query at all and the defaults apply.
//! That is why it survived: the tests that exercised these endpoints never
//! passed pagination, and nothing in `cargo check` can see it.
//!
//! Found live on 2026-08-24 in the e2e stack: six pakaian-dinas endpoints
//! answering 400 to `?page=1&per_page=100`.
//!
//! The boundary, measured rather than assumed:
//!
//! | field position                        | numeric field |
//! |---------------------------------------|---------------|
//! | inside the `#[serde(flatten)]`ed type | **400**       |
//! | sibling of the flattened field        | fine          |
//! | struct with no `flatten` at all       | fine          |
//!
//! `Option<T>` and `#[serde(default)]` make no difference — both still fail.
//!
//! Attach these to the numeric fields of any type that gets flattened into a
//! `Query<…>` extractor. They accept the JSON form too, so the same struct
//! stays usable in a request body or a config file.

use std::fmt;
use std::str::FromStr;

use serde::de::{self, Deserializer, Visitor};

macro_rules! tolerant_int {
    ($fn_name:ident, $t:ty) => {
        /// Deserialize a number that may arrive as text (query string) or as a
        /// real number (JSON). See the module docs for why this is needed.
        pub fn $fn_name<'de, D>(deserializer: D) -> Result<$t, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = $t;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "a {} or its decimal string form", stringify!($t))
                }

                fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                    <$t>::try_from(v).map_err(|_| E::custom(format!("{v} is out of range")))
                }

                fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                    <$t>::try_from(v).map_err(|_| E::custom(format!("{v} is out of range")))
                }

                fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                    // Trim: `?page= 2` survives some proxies and clients.
                    <$t>::from_str(v.trim())
                        .map_err(|_| E::custom(format!("invalid {}: {v:?}", stringify!($t))))
                }
            }

            // `deserialize_any` is required, not a shortcut: the buffered
            // `Content` only answers self-describing requests, so asking for
            // `deserialize_i32` here would fail exactly the same way.
            deserializer.deserialize_any(V)
        }
    };
}

tolerant_int!(de_i32, i32);
tolerant_int!(de_u32, u32);
tolerant_int!(de_i64, i64);
tolerant_int!(de_u64, u64);
tolerant_int!(de_usize, usize);

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Pagination {
        #[serde(default = "one", deserialize_with = "de_i32")]
        page: i32,
        #[serde(default = "twenty", deserialize_with = "de_i32")]
        per_page: i32,
        category: Option<String>,
    }
    fn one() -> i32 {
        1
    }
    fn twenty() -> i32 {
        20
    }

    /// The shape that was broken: a Query DTO flattening a paginator.
    #[derive(Debug, Deserialize)]
    struct Flattened {
        #[serde(flatten)]
        pagination: Pagination,
        jenis_id: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct Unsigned {
        #[serde(deserialize_with = "de_u32")]
        limit: u32,
    }

    #[derive(Debug, Deserialize)]
    struct UnsignedFlattened {
        #[serde(flatten)]
        inner: Unsigned,
    }

    #[test]
    fn flattened_pagination_accepts_a_query_string() {
        let q: Flattened = serde_urlencoded::from_str("page=1&per_page=100").unwrap();
        assert_eq!((q.pagination.page, q.pagination.per_page), (1, 100));
    }

    #[test]
    fn flattened_u32_accepts_a_query_string() {
        let q: UnsignedFlattened = serde_urlencoded::from_str("limit=50").unwrap();
        assert_eq!(q.inner.limit, 50);
    }

    #[test]
    fn defaults_still_apply_when_nothing_is_sent() {
        let q: Flattened = serde_urlencoded::from_str("").unwrap();
        assert_eq!((q.pagination.page, q.pagination.per_page), (1, 20));
    }

    #[test]
    fn sibling_fields_of_the_flattened_one_still_work() {
        let q: Flattened = serde_urlencoded::from_str("per_page=200&jenis_id=abc").unwrap();
        assert_eq!(q.pagination.per_page, 200);
        assert_eq!(q.jenis_id.as_deref(), Some("abc"));
        assert_eq!(q.pagination.category, None);
    }

    /// The other direction: loosening the type must not start accepting
    /// garbage. A bad page is still a 400, just with a clearer message.
    #[test]
    fn non_numeric_input_is_still_rejected() {
        assert!(serde_urlencoded::from_str::<Flattened>("page=notanumber").is_err());
        assert!(serde_urlencoded::from_str::<Flattened>("page=").is_err());
        assert!(serde_urlencoded::from_str::<UnsignedFlattened>("limit=-1").is_err());
    }

    /// The same struct is used outside query strings, so JSON numbers must
    /// keep working — that is what `visit_i64`/`visit_u64` are for.
    #[test]
    fn json_numbers_still_deserialize() {
        let q: Pagination = serde_json::from_str(r#"{"page":2,"per_page":50}"#).unwrap();
        assert_eq!((q.page, q.per_page), (2, 50));
    }

    /// Canary for the bug itself: without the tolerant deserializer this
    /// exact input fails. If serde ever fixes flatten upstream this test
    /// starts failing, which is the signal to delete this module.
    #[test]
    fn the_underlying_defect_is_still_present_in_serde() {
        #[derive(Debug, Deserialize)]
        struct Plain {
            #[serde(default)]
            page: i32,
        }
        #[derive(Debug, Deserialize)]
        struct PlainFlattened {
            #[serde(flatten)]
            inner: Plain,
        }
        // Without flatten the same field parses fine — the contrast IS the bug.
        let plain: Plain = serde_urlencoded::from_str("page=1").unwrap();
        assert_eq!(plain.page, 1);

        assert!(
            serde_urlencoded::from_str::<PlainFlattened>("page=1").is_err(),
            "serde now handles integers under flatten — lib_core::serde_query can be removed"
        );

        // And this is exactly how it stayed hidden: send no pagination and the
        // default applies, so the endpoint answers 200 and looks healthy.
        let empty: PlainFlattened = serde_urlencoded::from_str("").unwrap();
        assert_eq!(empty.inner.page, 0);
    }
}
