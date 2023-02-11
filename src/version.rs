use nom::bytes::complete::take_while1;
use nom::combinator::{eof, map_res, opt};
use nom::error::ErrorKind;
use nom::multi::separated_list1;
use nom::sequence::Tuple;
use nom::Parser;

use nom::{bytes::complete::tag, IResult};

fn parse_version(input: &str) -> IResult<&str, (bool, usize, Option<usize>, Option<usize>)> {
    let read_number = take_while1(|c: char| c.is_digit(10));
    let parse_number = map_res(read_number, |s: &str| s.parse::<usize>());
    let version_chunks = separated_list1(tag("."), parse_number);
    let wildchar = opt(tag(".").and(tag("*")));
    let (o, (chunks, wc, _)) = (version_chunks, wildchar, eof).parse(input)?;
    if chunks.len() > 3 {
        let error = nom::error::Error::new(input, ErrorKind::Eof);
        return Err(nom::Err::Error(error));
    }
    let result = (
        wc.is_some(),
        chunks[0],
        chunks.get(1).cloned(),
        chunks.get(2).cloned(),
    );
    Ok((o, result))
}

pub struct Version {
    major: usize,
    minor: usize,
    patch: usize,
}

pub enum Constraint {
    RangeInclusive(Version, Version),
    RangeExclusive(Version, Version),
}

impl Version {
    pub const MIN: Version = Version {
        major: 0,
        minor: 0,
        patch: 0,
    };
    pub const MAX: Version = Version {
        major: usize::MAX,
        minor: usize::MAX,
        patch: usize::MAX,
    };

    /// Parse version with `<major>.<minor>.<patch>` format. Will use [`nom`] to parse the version
    /// information.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to parsed version string
    pub fn parse(version: &str) -> Result<Self, nom::Err<nom::error::Error<&str>>> {
        let (_, (_, major, minor, patch)) = parse_version(version)?;
        Ok(Self {
            major,
            minor: minor.unwrap_or(0),
            patch: patch.unwrap_or(0),
        })
    }
}

impl Constraint {
    /// Pares version with constraints from `version`. Uses [Cargo Dependency] version format.
    ///
    /// [Cargo Dependency]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to parse version constraints.
    pub fn parse(version: &str) -> Result<Self, nom::Err<nom::error::Error<&str>>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{Constraint, Version};

    macro_rules! assert_version {
        ($version:ident, $major:expr,$minor:expr,$patch:expr) => {
            let msg = format!(
                "expected {}.{}.{} but got {}.{}.{}",
                $major, $minor, $patch, $version.major, $version.minor, $version.patch
            );
            assert_eq!($major, $version.major, "{msg}");
            assert_eq!($minor, $version.minor, "{msg}");
            assert_eq!($patch, $version.patch, "{msg}");
        };
    }

    macro_rules! test_version {
        ($name:ident, $version:literal => $major:expr,$minor:expr,$patch:expr) => {
            #[test]
            pub fn $name() {
                let version = Version::parse($version).unwrap();
                assert_version!(version, $major, $minor, $patch);
            }
        };
    }

    macro_rules! test_constraint {
        ($name:ident, $constraint:literal => [$mal:expr,$mil:expr,$pal:expr; $mau:expr,$miu:expr,$pau:expr]) => {
            #[test]
            pub fn $name() {
                let constraint = Constraint::parse($constraint).unwrap();

                match constraint {
                    Constraint::RangeExclusive(lower, upper) => {
                        assert_version!(lower, $mal, $mil, $pal);
                        assert_version!(upper, $mau, $miu, $pau);
                    }
                    _ => unreachable!(),
                }
            }
        };
    }

    test_version!(version_full, "1.2.3" => 1,2,3);
    test_version!(version_major_minor, "1.2" => 1,2,0);
    test_version!(version_major, "1" => 1,0,0);
    test_version!(version_major_minor_wildchar, "1.2.*" => 1,2,0);
    test_version!(version_major_wildchar, "1.*" => 1,0,0);

    #[test]
    pub fn version_extra() {
        let version = Version::parse("1.2.3.4");
        assert!(version.is_err())
    }

    test_constraint!(constraint_caret_full, "^1.2.3" => [1,2,3; 2,0,0]);
    test_constraint!(constraint_caret_major_minor, "^1.2" => [1,2,0; 2,0,0]);
    test_constraint!(constraint_caret_major, "^1" => [1,0,0; 2,0,0]);
    test_constraint!(constraint_caret_beta_full, "^0.2.3" => [0,2,3; 0,3,0]);
    test_constraint!(constraint_caret_beta_minor, "^0.2" => [0,2,0; 0,3,0]);
    test_constraint!(constraint_caret_alpha, "^0.0.3" => [0,0,3; 0,0,4]);
    test_constraint!(constraint_caret_0_0, "^0.0" => [0,0,0; 0,1,0]);
    test_constraint!(constraint_caret_0, "^0" => [0,0,0; 1,0,0]);

    test_constraint!(constraint_tilde_full, "~1.2.3" => [1,2,3; 1,3,0]);
    test_constraint!(constraint_tilde_major_minor, "~1.2" => [1,2,0; 1,3,0]);
    test_constraint!(constraint_tilde_major, "~1" => [1,0,0; 2,0,0]);

    test_constraint!(constraint_wildchar_major_minor, "1.2.*" => [1,2,0; 1,3,0]);
    test_constraint!(constraint_wildchar_major, "1.*" => [1,0,0; 2,0,0]);
    test_constraint!(constraint_wildchar, "*" => [0,0,0; usize::MAX,usize::MAX,usize::MAX]);
}
