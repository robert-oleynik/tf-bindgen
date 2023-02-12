use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::{take_while, take_while1};
use nom::combinator::{eof, map_res, opt, verify};
use nom::error::ErrorKind;
use nom::multi::separated_list1;
use nom::sequence::{preceded, Tuple};
use nom::Parser;

use nom::{bytes::complete::tag, IResult};

/// [`nom`] based parser for parsing [`Version`] struct.
pub fn parse_version(input: &str) -> IResult<&str, (bool, usize, Version)> {
    let read_number = take_while1(|ch: char| ch.is_ascii_digit());
    let parse_number = map_res(read_number, |s: &str| s.parse::<usize>());
    let version_chunks = separated_list1(tag("."), parse_number);
    let wildchar = opt(tag(".").and(tag("*")));
    let (o, (chunks, wc)) = (version_chunks, wildchar).parse(input)?;
    if chunks.len() > 3 {
        let error = nom::error::Error::new(input, ErrorKind::TooLarge);
        return Err(nom::Err::Error(error));
    }
    let result = (
        wc.is_some(),
        chunks.len(),
        Version {
            major: chunks[0],
            minor: chunks.get(1).cloned().unwrap_or(0),
            patch: chunks.get(2).cloned().unwrap_or(0),
        },
    );
    Ok((o, result))
}

fn caret_version(version: Version, depth: usize) -> Vec<Constraint> {
    let upper = if version.major >= 1 {
        version.next_major()
    } else if version.minor >= 1 {
        version.next_minor()
    } else {
        match depth {
            1 => version.next_major(),
            2 => version.next_minor(),
            3 => version.next_patch(),
            _ => unreachable!(),
        }
    };
    vec![
        Constraint::GreaterEquals(version),
        Constraint::LessThan(upper),
    ]
}

/// [`nom`] based parser for parsing [`Constraints`] structs.
pub fn parse_constraint(input: &str) -> IResult<&str, Vec<Constraint>> {
    let whitespace = take_while(char::is_whitespace);
    let caret_parser = verify(preceded(opt(tag("^")), parse_version), |(wc, _, _)| !wc)
        .map(|(_, depth, version)| caret_version(version, depth));
    let tilde_parser =
        verify(preceded(tag("~"), parse_version), |(wc, _, _)| !wc).map(|(_, depth, version)| {
            let upper = match depth {
                1 => version.next_major(),
                2 | 3 => version.next_minor(),
                _ => unreachable!(),
            };
            vec![
                Constraint::GreaterEquals(version),
                Constraint::LessThan(upper),
            ]
        });
    let wildchar1_parser = verify(parse_version, |(wc, _, _)| *wc).map(|(_, depth, version)| {
        let upper = match depth {
            1 => version.next_major(),
            2 => version.next_minor(),
            _ => unreachable!(),
        };
        vec![
            Constraint::GreaterEquals(version),
            Constraint::LessThan(upper),
        ]
    });
    let wildchar2_parser = tag("*").map(|_| {
        vec![
            Constraint::GreaterEquals(Version::MIN),
            Constraint::LessThan(Version::MAX),
        ]
    });
    let compare = verify(
        alt((tag(">="), tag("<="), tag("="), tag(">"), tag("<")))
            .and(take_while(char::is_whitespace))
            .and(parse_version),
        |(_, (wc, _, _))| !wc,
    )
    .map(|((tag, _), (_, _, version))| match tag {
        ">=" => Constraint::GreaterEquals(version),
        "<=" => Constraint::LessEquals(version),
        "=" => Constraint::Equals(version),
        ">" => Constraint::GreaterThan(version),
        "<" => Constraint::LessThan(version),
        _ => unreachable!(),
    })
    .map(|constraint| vec![constraint]);

    let constraint_param = tilde_parser
        .or(wildchar2_parser)
        .or(wildchar1_parser)
        .or(compare)
        .or(caret_parser);
    let parser = separated_list1(tag(","), whitespace.and(constraint_param)).map(|constraints| {
        constraints
            .into_iter()
            .flat_map(|(_, c)| c.into_iter())
            .collect::<Vec<_>>()
    });
    let (_, (constraints, _)) = (parser, eof).parse(input)?;
    Ok(("", constraints))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    major: usize,
    minor: usize,
    patch: usize,
}

pub type Constraints = Vec<Constraint>;

#[derive(Clone, Debug)]
pub enum Constraint {
    LessThan(Version),
    GreaterThan(Version),
    Equals(Version),
    LessEquals(Version),
    GreaterEquals(Version),
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

    pub fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse version with `<major>.<minor>.<patch>` format. Will use [`nom`] to parse the version
    /// information.
    ///
    /// # Errors
    ///
    /// Will return `Err` if failed to parsed version string
    pub fn parse(version: &str) -> Result<Self, nom::Err<nom::error::Error<&str>>> {
        let (_, ((_, _, version), _)) = (parse_version, eof).parse(version)?;
        Ok(version)
    }

    /// Increment this version to the next major version.
    ///
    /// # Example
    ///
    /// ```rust
    /// use terraform_bindgen::version::Version;
    ///
    /// let version = Version::new(1, 2, 3);
    /// let next = version.next_major();
    /// assert_eq!(next.major(), 2);
    /// assert_eq!(next.minor(), 0);
    /// assert_eq!(next.patch(), 0);
    /// ```
    pub fn next_major(&self) -> Version {
        Version {
            major: self.major + 1,
            minor: 0,
            patch: 0,
        }
    }

    /// Increment this version to the next minor version.
    ///
    /// # Example
    ///
    /// ```rust
    /// use terraform_bindgen::version::Version;
    ///
    /// let version = Version::new(1, 2, 3);
    /// let next = version.next_minor();
    /// assert_eq!(next.major(), 1);
    /// assert_eq!(next.minor(), 3);
    /// assert_eq!(next.patch(), 0);
    /// ```
    pub fn next_minor(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
        }
    }

    /// Increment this version to the next minor version.
    ///
    /// # Example
    ///
    /// ```rust
    /// use terraform_bindgen::version::Version;
    ///
    /// let version = Version::new(1, 2, 3);
    /// let next = version.next_patch();
    /// assert_eq!(next.major(), 1);
    /// assert_eq!(next.minor(), 2);
    /// assert_eq!(next.patch(), 4);
    /// ```
    pub fn next_patch(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
        }
    }

    pub fn major(&self) -> usize {
        self.major
    }
    pub fn minor(&self) -> usize {
        self.minor
    }
    pub fn patch(&self) -> usize {
        self.patch
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
    pub fn parse(version: &str) -> Result<Vec<Constraint>, nom::Err<nom::error::Error<&str>>> {
        let (_, constraints) = parse_constraint(version)?;
        Ok(constraints)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.major.to_string())?;
        f.write_str(".")?;
        f.write_str(&self.minor.to_string())?;
        f.write_str(".")?;
        f.write_str(&self.patch.to_string())
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constraint::LessThan(v) => f.write_fmt(format_args!("<{v}")),
            Constraint::GreaterThan(v) => f.write_fmt(format_args!(">{v}")),
            Constraint::Equals(v) => f.write_fmt(format_args!("={v}")),
            Constraint::LessEquals(v) => f.write_fmt(format_args!("<={v}")),
            Constraint::GreaterEquals(v) => f.write_fmt(format_args!(">={v}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Constraint, Version};

    macro_rules! assert_version {
        ($version:expr, $major:expr,$minor:expr,$patch:expr) => {{
            let msg = format!(
                "expected {}.{}.{} but got {}.{}.{}",
                $major, $minor, $patch, $version.major, $version.minor, $version.patch
            );
            assert_eq!($major, $version.major, "{msg}");
            assert_eq!($minor, $version.minor, "{msg}");
            assert_eq!($patch, $version.patch, "{msg}");
        }};
    }

    macro_rules! assert_constraint {
        ($constraint:expr, = $major:tt:$minor:tt:$patch:tt) => {
            if let Constraint::Equals(version) = $constraint {
                assert_version!(version, $major, $minor, $patch)
            } else {
                panic!()
            }
        };
        ($constraint:expr, < $major:tt:$minor:tt:$patch:tt) => {
            if let Constraint::LessThan(version) = $constraint {
                assert_version!(version, $major, $minor, $patch)
            } else {
                panic!()
            }
        };
        ($constraint:expr, > $major:tt:$minor:tt:$patch:tt) => {
            if let Constraint::GreaterThan(version) = $constraint {
                assert_version!(version, $major, $minor, $patch)
            } else {
                panic!()
            }
        };
        ($constraint:expr, <= $major:tt:$minor:tt:$patch:tt) => {
            if let Constraint::LessEquals(version) = $constraint {
                assert_version!(version, $major, $minor, $patch)
            } else {
                panic!()
            }
        };
        ($constraint:expr, >= $major:tt:$minor:tt:$patch:tt) => {
            if let Constraint::GreaterEquals(version) = $constraint {
                assert_version!(version, $major, $minor, $patch)
            } else {
                panic!()
            }
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
        ($name:ident, $constraint:literal => $($op:tt $major:tt:$minor:tt:$patch:tt),*) => {
            #[test]
            pub fn $name() {
                let mut constraints = Constraint::parse($constraint).unwrap();

				$(
					let constraint = constraints.remove(0);
					assert_constraint!(constraint, $op $major:$minor:$patch);
				)*
				assert_eq!(0, constraints.len());
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

    test_constraint!(constraint_caret_full, "^1.2.3" => >= 1:2:3, < 2:0:0);
    test_constraint!(constraint_caret_major_minor, "^1.2" => >=1:2:0, < 2:0:0);
    test_constraint!(constraint_caret_major, "^1" => >=1:0:0, < 2:0:0);
    test_constraint!(constraint_caret_beta_full, "^0.2.3" => >=0:2:3, < 0:3:0);
    test_constraint!(constraint_caret_beta_minor, "^0.2" => >=0:2:0, < 0:3:0);
    test_constraint!(constraint_caret_alpha, "^0.0.3" => >=0:0:3, < 0:0:4);
    test_constraint!(constraint_caret_0_0, "^0.0" => >=0:0:0, < 0:1:0);
    test_constraint!(constraint_caret_0, "^0" => >=0:0:0, < 1:0:0);

    test_constraint!(constraint_full, "1.2.3" => >= 1:2:3, < 2:0:0);
    test_constraint!(constraint_major_minor, "1.2" => >=1:2:0, < 2:0:0);
    test_constraint!(constraint_major, "1" => >=1:0:0, < 2:0:0);
    test_constraint!(constraint_beta_full, "0.2.3" => >=0:2:3, < 0:3:0);
    test_constraint!(constraint_beta_minor, "0.2" => >=0:2:0, < 0:3:0);
    test_constraint!(constraint_alpha, "0.0.3" => >=0:0:3, < 0:0:4);
    test_constraint!(constraint_0_0, "0.0" => >=0:0:0, < 0:1:0);
    test_constraint!(constraint_0, "0" => >=0:0:0, < 1:0:0);

    test_constraint!(constraint_tilde_full, "~1.2.3" => >= 1:2:3, < 1:3:0);
    test_constraint!(constraint_tilde_major_minor, "~1.2" => >= 1:2:0, < 1:3:0);
    test_constraint!(constraint_tilde_major, "~1" => >= 1:0:0, < 2:0:0);

    test_constraint!(constraint_wildchar_major_minor, "1.2.*" => >= 1:2:0, < 1:3:0);
    test_constraint!(constraint_wildchar_major, "1.*" => >= 1:0:0, < 2:0:0);

    test_constraint!(constraint_equals, "=1.2.3" => =1:2:3);
    test_constraint!(constraint_greater_than, ">1.2.3" => >1:2:3);
    test_constraint!(constraint_greater_equals, ">=1.2.3" => >=1:2:3);
    test_constraint!(constraint_less_than, "<1.2.3" => <1:2:3);
    test_constraint!(constraint_less_equals, "<=1.2.3" => <=1:2:3);

    test_constraint!(constraint_multi, ">=1.2.3, <2.0.0" => >=1:2:3, <2:0:0);
    test_constraint!(constraint_whitespace, ">= 1.2.3" => >=1:2:3);

    #[test]
    pub fn constraint_wildchar() {
        let constraint = Constraint::parse("*").unwrap();
        assert_eq!(2, constraint.len());
        if let Constraint::GreaterEquals(version) = &constraint[0] {
            assert_eq!(Version::MIN, *version)
        } else {
            unimplemented!()
        }
        if let Constraint::LessThan(version) = &constraint[1] {
            assert_eq!(Version::MAX, *version)
        } else {
            unimplemented!()
        }
    }
}
