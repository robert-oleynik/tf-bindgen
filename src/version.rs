use std::num::ParseIntError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Version has to many chunks. Expexted at max 3, but got {0}")]
    ToManyChunks(usize),
    #[error("{0}")]
    ParseVersionChunk(#[from] ParseIntError),
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
    /// Parse version with `<major>.<minor>.<patch>` format
    pub fn parse(version: &str) -> Result<Version, Error> {
        let chunks: Vec<_> = version.split(".").collect();
        if chunks.len() > 3 {
            return Err(Error::ToManyChunks(chunks.len()));
        }
        let major_chunk = chunks[0];
        let minor_chunk = chunks.get(1).map(|minor| *minor);
        let patch_chunk = chunks.get(2).map(|minor| *minor);

        let major: usize = major_chunk.parse()?;
        let minor = match minor_chunk {
            Some("*") => 0,
            Some(num) => num.parse()?,
            None => 0,
        };
        let patch = match patch_chunk {
            Some("*") => 0,
            Some(num) => num.parse()?,
            None => 0,
        };

        Ok(Version {
            major,
            minor,
            patch,
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
    pub fn parse(version: &str) -> Result<Constraint, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{Constraint, Version};

    macro_rules! assert_version {
        ($version:ident, $major:expr,$minor:expr,$patch:expr) => {
            assert_eq!($version.major, $major);
            assert_eq!($version.minor, $minor);
            assert_eq!($version.patch, $patch);
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
