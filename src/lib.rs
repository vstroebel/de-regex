/*!

# De-Regex

This crate contains a library that deserializes a string into a struct based on a regular expression and serde.

## Example: Parse image dimension into struct

```rust
# fn main() -> Result<(), de_regex::Error> {
use serde::Deserialize;

#[derive(Deserialize)]
struct Dimension {
    width: u32,
    height: u32
}

let pattern = r"^(?P<width>\d+)x(?P<height>\d+)$";
let input = "800x600";

let dim: Dimension = de_regex::from_str(input, pattern)?;

assert_eq!(dim.width, 800);
assert_eq!(dim.height, 600);
# Ok(())
# }
```
## Supported data types

The following data types can be used as struct fields.

- **bool**: Supported values are `true` or `false` case insensitive<br>
            Example pattern: `^(?P<group_name>(?i)(true|false))$`

- **u8, u16, u32, u64**: Decimal values prefixed with an optional `+`<br>
            Example pattern: `^(?P<group_name>\+?\d+)$`

- **i8, i16, i32, i64**: Decimal values prefixed with an optional `+`<br>
            Example pattern: `^(?P<group_name>[-+]?\d+)$`

- **f32, f64**: See the documentation of the [FromStr](https://doc.rust-lang.org/std/primitive.f32.html#impl-FromStr) implementation of f32/f64 for the valid syntax<br>
            Example pattern for simple decimal floats: `^(?P<group_name>[-+]?\d+(\.\d*)?)$`

- **String**: A unicode (utf8) string value.<br>
            Example pattern: `^(?P<group_name>\w*)$`

- **Tuple struct**: A tuple struct with one field (New Type Idiom). The struct needs to implement ´Deserialize´:
    ```rust
      # use serde::Deserialize;
      #[derive(Deserialize)]
      struct NewType(i32);
    ```

- **Enum**: Enums with unit variants (No newtypes and variant fields). The enum needs to implement ´Deserialize´:
    ```rust
      # use serde::Deserialize;
      #[derive(Deserialize)]
      enum SomeEnum {
        Foo,
        #[serde(rename = "bar")]
        Bar,
        Baz(i32) // <- Will compile but is not available in matching because of newtype
      };
    ```

- **Option<>**: All types above can be used as an optional value

Other data types supported by `serde` might work but are not officially supported and tested.

### Words of wisdom

If your regular expression looks like a behemoth no mere mortal will ever understand, please reconsider using this crate
*/

/*
 * IMPLEMENTATION NOTES
 *
 * The implementation is based on two implementations of serde::de::Deserializer
 *
 * 1. struct Deserializer:
 *    The toplevel deserializer that implements the deserialization of "maps".
 *    The regular expressions are matched against the input here and passed
 *    into serde::de::value::MapDeserializer for further processing of the map/struct.
 *
 * 2. struct Value:
 *    Responsible to deserialize struct members.
 *    For most types parsing is based on std::str::FromStr
 *
 */

mod error;
mod de;

pub use error::Error;

use serde::Deserialize;
use regex::Regex;

/// Deserialize an input string into a struct.
///
/// # Example
/// ```rust
/// # fn main() -> Result<(), de_regex::Error> {
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Dimension {
///     width: u32,
///     height: u32
/// }
///
/// let pattern = r"^(?P<width>\d+)x(?P<height>\d+)$";
/// let input = "800x600";
///
/// let dim: Dimension = de_regex::from_str(input, pattern)?;
///
/// assert_eq!(dim.width, 800);
/// assert_eq!(dim.height, 600);
/// # Ok(())
/// # }
/// ```
pub fn from_str<'a, T>(input: &'a str, regex: &str) -> std::result::Result<T, Error> where T: Deserialize<'a> {
    let regex = Regex::new(&regex).map_err(Error::BadRegex)?;
    from_str_regex(input, regex)
}

/// Deserialize an input string into a struct.
///
/// # Example
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use serde::Deserialize;
/// use regex::Regex;
///
/// #[derive(Deserialize)]
/// struct Dimension {
///     width: u32,
///     height: u32
/// }
///
/// let pattern = Regex::new(r"^(?P<width>\d+)x(?P<height>\d+)$")?;
/// let input = "800x600";
///
/// let dim: Dimension = de_regex::from_str_regex(input, pattern)?;
///
/// assert_eq!(dim.width, 800);
/// assert_eq!(dim.height, 600);
/// # Ok(())
/// # }
/// ```
pub fn from_str_regex<'a, T>(input: &'a str, regex: Regex) -> std::result::Result<T, Error> where T: Deserialize<'a> {
    let mut deserializer = de::Deserializer::new(input, regex);
    T::deserialize(&mut deserializer)
}

#[cfg(test)]
mod test {
    use super::*;
    use super::error::Result;

    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        foo: u32,
        bar: i32,
    }

    #[test]
    fn test_simple() {
        let regex = r"^(?P<foo>\d*),(?P<bar>-?\d*)$";
        let input = "1,-2";
        let output: Test = from_str(input, regex).unwrap();

        assert_eq!(output, Test { foo: 1, bar: -2 });
    }

    #[test]
    fn test_bad_regex() {
        let regex = r"^(?P<foo\d*),(?P<bar>\d*)$";
        let input = "1,-2";
        let output: Result<Test> = from_str(input, regex);

        assert!(output.is_err());
    }

    #[test]
    fn test_bad_input() {
        let regex = r"^(?P<foo\d*),(?P<bar>\d*)$";
        let input = "";
        let output: Result<Test> = from_str(input, regex);

        assert!(output.is_err());
    }

    #[test]
    fn test_missing_group() {
        let regex = r"^(?P<foo>\d*)$";
        let input = "1";
        let output: Result<Test> = from_str(input, regex);

        assert!(output.is_err());
    }

    #[test]
    fn test_explicit_positive_int() {
        let regex = r"^(?P<foo>\+?\d*),(?P<bar>[-+]?\d*)$";
        let input = "+1,+2";
        let output: Test = from_str(input, regex).unwrap();

        assert_eq!(output, Test { foo: 1, bar: 2 });
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Test2 {
        f_bool: bool,
        f_u8: u8,
        f_u16: u16,
        f_u32: u32,
        f_u64: u64,
        f_i8: i8,
        f_i16: i16,
        f_i32: i32,
        f_i64: i64,
        f_f32: f32,
        f_f64: f64,
        f_str: String,
    }

    const TEST2_PATTERN: &'static str = r"^(?P<f_bool>\w*),(?P<f_u8>\d*),(?P<f_u16>\d*),(?P<f_u32>\d*),(?P<f_u64>\d*),(?P<f_i8>-?\d*),(?P<f_i16>-?\d*),(?P<f_i32>-?\d*),(?P<f_i64>-?\d*),(?P<f_f32>-?\d*\.?\d?),(?P<f_f64>-?\d*\.?\d?),(?P<f_str>\w*)$";

    #[test]
    fn test_supported_types() {
        let input = "true,1,2,3,4,-1,-2,-3,-4,1.0,-1.0,foobar";
        let output: Test2 = from_str(input, TEST2_PATTERN).unwrap();

        assert_eq!(output, Test2 {
            f_bool: true,
            f_u8: 1,
            f_u16: 2,
            f_u32: 3,
            f_u64: 4,
            f_i8: -1,
            f_i16: -2,
            f_i32: -3,
            f_i64: -4,
            f_f32: 1.0,
            f_f64: -1.0,
            f_str: "foobar".to_owned(),
        });
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Test3 {
        foo: Option<u32>,
        bar: Option<i32>,
    }

    #[test]
    fn test_option() {
        let regex = r"^(?P<foo>\d*),(?P<bar>-?\d*)$";
        let input = "1,-2";
        let output: Test3 = from_str(input, regex).unwrap();

        assert_eq!(output, Test3 { foo: Some(1), bar: Some(-2) });
    }

    #[test]
    fn test_option_none() {
        let regex = r"^(?P<foo>\d*),(?P<bar>-?\d*)$";
        let input = ",";
        let output: Test3 = from_str(input, regex).unwrap();

        assert_eq!(output, Test3 { foo: None, bar: None });
    }

    #[test]
    fn test_bool() {
        #[derive(Deserialize)]
        struct TestBool {
            v: bool,
        }

        let regex = r"^(?P<v>(?i)(true|false))$";

        assert!(from_str::<TestBool>("true", regex).unwrap().v);
        assert!(!from_str::<TestBool>("false", regex).unwrap().v);
        assert!(from_str::<TestBool>("TRUE", regex).unwrap().v);
        assert!(!from_str::<TestBool>("FALSE", regex).unwrap().v);
        assert!(from_str::<TestBool>("trUE", regex).unwrap().v);
        assert!(!from_str::<TestBool>("FAlse", regex).unwrap().v);

        let regex = r"^(?P<v>\w*)$";

        assert!(from_str::<TestBool>("SOMETHING ELSE", regex).is_err());
        assert!(from_str::<TestBool>("", regex).is_err());
    }

    #[test]
    fn test_uint() {
        #[derive(Deserialize)]
        struct TestUInt {
            v: u32,
        }

        let regex = r"^(?P<v>\+?\d+)$";

        assert_eq!(123, from_str::<TestUInt>("123", regex).unwrap().v);
        assert_eq!(123, from_str::<TestUInt>("+123", regex).unwrap().v);
        assert!(from_str::<TestUInt>("-123", regex).is_err());
    }

    #[test]
    fn test_int() {
        #[derive(Deserialize)]
        struct TestInt {
            v: i32,
        }

        let regex = r"^(?P<v>[-+]?\d+)$";

        assert_eq!(123, from_str::<TestInt>("123", regex).unwrap().v);
        assert_eq!(123, from_str::<TestInt>("+123", regex).unwrap().v);
        assert_eq!(-123, from_str::<TestInt>("-123", regex).unwrap().v);
        assert!(from_str::<TestInt>("#123", regex).is_err());
    }

    #[test]
    fn test_float() {
        #[derive(Deserialize)]
        struct TestFloat {
            v: f32,
        }

        let regex = r"^(?P<v>[-+]?\d+(\.\d*)?)$";

        assert_eq!(123.0, from_str::<TestFloat>("123", regex).unwrap().v);
        assert_eq!(1.23, from_str::<TestFloat>("1.23", regex).unwrap().v);
        assert_eq!(-123.0, from_str::<TestFloat>("-123", regex).unwrap().v);
        assert_eq!(-1.23, from_str::<TestFloat>("-1.23", regex).unwrap().v);

        assert!(from_str::<TestFloat>("#123", regex).is_err());
    }

    #[test]
    fn test_bad_value_error() {
        let regex = r"^(?P<foo>\w*),(?P<bar>-?\d*)$";
        let input = "aaa1,-2";
        let output: Result<Test> = from_str(input, regex);

        assert!(matches!(output, Err(Error::BadValue{..})), "Expected Error::BadValue got {:?}", output);
    }

    #[test]
    fn test_newtype() {
        #[derive(Deserialize)]
        struct NewType(i32);

        #[derive(Deserialize)]
        struct Test {
            v: NewType,
        }

        let regex = r"^(?P<v>[-+]?\d+)$";

        assert_eq!(123, from_str::<Test>("123", regex).unwrap().v.0);
        assert_eq!(-123, from_str::<Test>("-123", regex).unwrap().v.0);
        assert!(from_str::<Test>("#123", regex).is_err());
    }

    #[test]
    fn test_enum_simple() {
        #[allow(dead_code)]
        #[derive(Deserialize, Debug, PartialEq)]
        enum TestEnum {
            Foo,
            #[serde(rename = "bar")]
            Bar,
            #[serde(skip)]
            Baz(i32),
        }

        #[derive(Deserialize)]
        struct Test {
            v: TestEnum,
        }

        let regex = r"^(?P<v>[-+]?\w+)$";

        assert_eq!(TestEnum::Foo, from_str::<Test>("Foo", regex).unwrap().v);
        assert_eq!(TestEnum::Bar, from_str::<Test>("bar", regex).unwrap().v);
        assert!(from_str::<Test>("foo", regex).is_err());
        assert!(from_str::<Test>("Baz", regex).is_err());
    }
}
