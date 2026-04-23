use super::{
    BoolGenerator, BoxedGenerator, CharactersGenerator, DurationGenerator, FloatGenerator,
    Generator, HashMapGenerator, IntegerGenerator, OptionalGenerator, TextGenerator, VecGenerator,
    booleans, characters, collections::ArrayGenerator, durations, floats, hashmaps, integers,
    optional, text, vecs,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::time::Duration;

fn path_segment_generator() -> BoxedGenerator<'static, String> {
    use super::{just, one_of};

    one_of([
        text().max_size(12).boxed(),
        just(".".to_string()).boxed(),
        just("..".to_string()).boxed(),
        just(String::new()).boxed(),
        text()
            .min_size(1)
            .max_size(8)
            .map(|segment| format!(".{segment}"))
            .boxed(),
        text().min_size(200).max_size(255).boxed(),
        just(" ".to_string()).boxed(),
    ])
    .boxed()
}

fn build_pathbuf(segments: Vec<String>, absolute: bool) -> PathBuf {
    let path: PathBuf = segments.iter().collect();
    if segments.is_empty() || !absolute {
        return path;
    }

    absolute_path_root().join(path)
}

fn absolute_path_root() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from("C:\\")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/")
    }
}

/// Trait for types that have a default generator.
///
/// This is used by derive macros to automatically generate values for fields.
pub trait DefaultGenerator: Sized {
    type Generator: super::Generator<Self> + 'static;
    fn default_generator() -> Self::Generator;
}

/// Create a generator for a type using its default generator.
///
/// This is the primary way to get a generator for types that implement
/// [`DefaultGenerator`], including types with `#[derive(DefaultGenerator)]`.
///
/// # Example
///
/// ```no_run
/// use hegel::generators::{self as gs, DefaultGenerator};
/// use hegel::DefaultGenerator;
///
/// #[derive(DefaultGenerator, Debug)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// #[hegel::test]
/// fn my_test(tc: hegel::TestCase) {
///     // Generate with defaults
///     let person: Person = tc.draw(gs::default::<Person>());
///
///     // Customize field generators
///     let person: Person = tc.draw(gs::default::<Person>()
///         .age(gs::integers().min_value(0).max_value(120)));
/// }
/// ```
pub fn default<T: DefaultGenerator>() -> BoxedGenerator<'static, T> {
    T::default_generator().boxed()
}

impl DefaultGenerator for bool {
    type Generator = BoolGenerator;
    fn default_generator() -> Self::Generator {
        booleans()
    }
}

impl DefaultGenerator for String {
    type Generator = TextGenerator;
    fn default_generator() -> Self::Generator {
        text()
    }
}

impl DefaultGenerator for char {
    type Generator = CharactersGenerator;
    fn default_generator() -> Self::Generator {
        characters()
    }
}

impl DefaultGenerator for i8 {
    type Generator = IntegerGenerator<i8>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for i16 {
    type Generator = IntegerGenerator<i16>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for i32 {
    type Generator = IntegerGenerator<i32>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for i64 {
    type Generator = IntegerGenerator<i64>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for u8 {
    type Generator = IntegerGenerator<u8>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for u16 {
    type Generator = IntegerGenerator<u16>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for u32 {
    type Generator = IntegerGenerator<u32>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for u64 {
    type Generator = IntegerGenerator<u64>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for i128 {
    type Generator = IntegerGenerator<i128>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for u128 {
    type Generator = IntegerGenerator<u128>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for isize {
    type Generator = IntegerGenerator<isize>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for usize {
    type Generator = IntegerGenerator<usize>;
    fn default_generator() -> Self::Generator {
        integers()
    }
}

impl DefaultGenerator for f32 {
    type Generator = FloatGenerator<f32>;
    fn default_generator() -> Self::Generator {
        floats()
    }
}

impl DefaultGenerator for f64 {
    type Generator = FloatGenerator<f64>;
    fn default_generator() -> Self::Generator {
        floats()
    }
}

impl<T: DefaultGenerator + 'static> DefaultGenerator for Option<T>
where
    T::Generator: Send + Sync,
{
    type Generator = OptionalGenerator<T::Generator, T>;
    fn default_generator() -> Self::Generator {
        optional(T::default_generator())
    }
}

impl<T: DefaultGenerator + 'static> DefaultGenerator for Vec<T>
where
    T::Generator: Send + Sync,
{
    type Generator = VecGenerator<T::Generator, T>;
    fn default_generator() -> Self::Generator {
        vecs(T::default_generator())
    }
}

impl<T: DefaultGenerator + 'static, const N: usize> DefaultGenerator for [T; N]
where
    T::Generator: Send + Sync,
{
    type Generator = ArrayGenerator<T::Generator, T, N>;
    fn default_generator() -> Self::Generator {
        ArrayGenerator::new(T::default_generator())
    }
}

impl DefaultGenerator for Duration {
    type Generator = DurationGenerator;
    fn default_generator() -> Self::Generator {
        durations()
    }
}

/// Generates filesystem paths covering common edge cases.
///
/// The generator produces paths from 0 to 8 segments joined with the
/// platform path separator. Segments are drawn from a mix of:
///
/// - Short alphanumeric text (the common case)
/// - `.` and `..` (traversal)
/// - Empty string (consecutive separators / trailing slash)
/// - Dot-prefixed names (`.hidden`)
/// - Long names near the typical 255-byte NAME_MAX limit
/// - Whitespace-only names
///
/// Roughly 10% of generated paths are absolute (prefixed with `/` on
/// Unix, `C:\` on Windows). An empty segment vector produces `""`,
/// the empty path.
impl DefaultGenerator for PathBuf {
    type Generator = BoxedGenerator<'static, PathBuf>;
    fn default_generator() -> Self::Generator {
        use super::sampled_from;

        sampled_from(&[
            false, false, false, false, false, false, false, false, false, true,
        ])
        .flat_map(|absolute| {
            vecs(path_segment_generator())
                .max_size(8)
                .map(move |segments| build_pathbuf(segments, absolute))
        })
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::{absolute_path_root, build_pathbuf};
    use std::path::PathBuf;

    #[test]
    fn empty_absolute_path_stays_empty() {
        assert_eq!(build_pathbuf(Vec::new(), true), PathBuf::new());
    }

    #[test]
    fn relative_path_stays_relative() {
        assert_eq!(
            build_pathbuf(vec!["alpha".to_string(), "beta".to_string()], false),
            PathBuf::from("alpha").join("beta")
        );
    }

    #[test]
    fn absolute_path_uses_platform_root() {
        assert_eq!(
            build_pathbuf(vec!["alpha".to_string(), "beta".to_string()], true),
            absolute_path_root().join("alpha").join("beta")
        );
    }
}

impl<K: DefaultGenerator + 'static, V: DefaultGenerator + 'static> DefaultGenerator
    for HashMap<K, V>
where
    K: Eq + Hash,
    K::Generator: Send + Sync,
    V::Generator: Send + Sync,
{
    type Generator = HashMapGenerator<K::Generator, V::Generator, K, V>;
    fn default_generator() -> Self::Generator {
        hashmaps(K::default_generator(), V::default_generator())
    }
}

/// Derive a generator for a struct type defined externally.
///
/// This macro creates a hidden generator struct with builder methods for each field,
/// and implements [`DefaultGenerator`] for the type
/// so it can be used with [`default`].
///
/// # Example
///
/// ```ignore
/// // Externally defined
/// pub struct Person {
///     pub name: String,
///     pub age: u32,
/// }
///
/// // In your tests:
/// use hegel::derive_generator;
/// use hegel::generators::{self as gs, DefaultGenerator, Generator};
/// use production_crate::Person;
///
/// derive_generator!(Person {
///     name: String,
///     age: u32,
/// });
///
/// // default now supports Person:
/// let generator = gs::default::<Person>()
///     .name(gs::from_regex("[A-Z][a-z]+"))
///     .age(gs::integers::<u32>().min_value(0).max_value(120));
///
/// let person: Person = tc.draw(generator);
/// ```
#[macro_export]
macro_rules! derive_generator {
    ($struct_name:ident { $($field_name:ident : $field_type:ty),* $(,)? }) => {
        const _: () = {
            $crate::paste::paste! {
                pub struct [<$struct_name Generator>]<'a> {
                    $(
                        $field_name: $crate::generators::BoxedGenerator<'a, $field_type>,
                    )*
                }

                impl<'a> [<$struct_name Generator>]<'a> {
                    pub fn new() -> Self
                    where
                        $($field_type: $crate::generators::DefaultGenerator,)*
                        $(<$field_type as $crate::generators::DefaultGenerator>::Generator: Send + Sync + 'a,)*
                    {
                        use $crate::generators::{DefaultGenerator, Generator};
                        Self {
                            $($field_name: <$field_type as DefaultGenerator>::default_generator().boxed(),)*
                        }
                    }

                    $(
                        pub fn $field_name<G>(mut self, generator: G) -> Self
                        where
                            G: $crate::generators::Generator<$field_type> + Send + Sync + 'a,
                        {
                            use $crate::generators::Generator;
                            self.$field_name = generator.boxed();
                            self
                        }
                    )*
                }

                impl<'a> Default for [<$struct_name Generator>]<'a>
                where
                    $($field_type: $crate::generators::DefaultGenerator,)*
                    $(<$field_type as $crate::generators::DefaultGenerator>::Generator: Send + Sync + 'a,)*
                {
                    fn default() -> Self {
                        Self::new()
                    }
                }

                impl<'a> $crate::generators::Generator<$struct_name> for [<$struct_name Generator>]<'a> {
                    fn do_draw(&self, __data: &$crate::TestCase) -> $struct_name {
                        use $crate::generators::Generator;
                        $struct_name {
                            $($field_name: self.$field_name.do_draw(__data),)*
                        }
                    }
                }

                impl $crate::generators::DefaultGenerator for $struct_name
                where
                    $($field_type: $crate::generators::DefaultGenerator,)*
                    $(<$field_type as $crate::generators::DefaultGenerator>::Generator: Send + Sync + 'static,)*
                {
                    type Generator = [<$struct_name Generator>]<'static>;
                    fn default_generator() -> Self::Generator {
                        [<$struct_name Generator>]::new()
                    }
                }
            }
        };
    };
}
