use std::collections::HashMap;
use std::path::{Path, PathBuf};

use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[cfg(test)]
mod tests {

    use crate::parse_package_use;
    #[test]
    fn it_works() {
        println!("Map: {:#?}", parse_package_use());
        assert!(false, true);
    }
}

#[derive(Clone, Copy, Debug)]
enum VersionConstraint {
	NoConstraint,
	LessThan,
	LessEqual,
	EqualTo,
	GreaterThan,
	GreaterEqual,
}

#[derive(Debug)]
enum UseFlagState {
	Enabled,
	Disabled,
}

#[derive(Debug)]
struct PortagePackage {
	name: String,
	version: Option<String>,
	use_flags: HashMap<String, UseFlagState>,
}

impl PortagePackage {
	fn from(s: &str) -> Result<Self, i32> {
		let (name, version) = get_package_name(s)?;
		Ok(Self {
			name: name.to_string(),
			version: version.map(|s| s.to_string()),
			use_flags: HashMap::new(),
		})
	}
}

#[derive(Debug)]
struct PackageConstraint {
	package: PortagePackage,
	version_constraint: Option<VersionConstraint>,
}

fn get_version_constraint(s: &str) -> Option<VersionConstraint> {
	if s.starts_with("<=") {
		Some(VersionConstraint::LessEqual)
	} else if s.starts_with("<") {
		Some(VersionConstraint::LessThan)
	} else if s.starts_with(">=") {
		Some(VersionConstraint::GreaterEqual)
	} else if s.starts_with(">") {
		Some(VersionConstraint::GreaterThan)
	} else if s.starts_with("=") {
		Some(VersionConstraint::EqualTo)
	} else {
		None
	}
}

fn get_package_name<'a>(s: &'a str) -> Result<(&'a str, Option<&'a str>), i32> {
	let br: &[_] = &['<', '>', '='];
	let s = s.trim_start_matches(br);
	let mut dash = false;
	let mut dash_ind = 0;
	for (i, ch) in s.char_indices() {
		if ch == '-' {
			dash = true;
			dash_ind = i;
			continue;
		}

		if ch.is_ascii_digit() && dash {
			return Ok((&s[..dash_ind], Some(&s[dash_ind+1..])));
		}

		dash = false;
	}
	return Ok((s, None));
}

fn parse_package_use() -> HashMap<String, PackageConstraint> {
	let path = Path::new("/etc/portage/package.use");

	let mut map = HashMap::new();

	if path.is_dir() {

	} else if path.is_file() {
		let f = File::open(path).unwrap();
		let f = BufReader::new(f);

		for line in f.lines() {
			let line = line.unwrap();
			let line = line.trim();
			let line = match line.find('#') {
				Some(s) => &line[..s],
				None => line,
			};
			if line.len() == 0 {
				continue;
			}

			let mut vec = Vec::new();
			for token in line.split(' ') {
				if token.len() == 0 {
					continue;
				}
				vec.push(token);
			}

			if vec.len() <= 1 {
				continue;
			}

			let version_constraint = get_version_constraint(vec[0]);
			let mut package = PortagePackage::from(vec[0]).unwrap();

			for useflag in &vec[1..] {
				if useflag.starts_with('-') {
					package.use_flags.insert(useflag[1..].to_string(), UseFlagState::Disabled);
				} else {
					package.use_flags.insert(useflag.to_string(), UseFlagState::Enabled);
				}
			}

			let package = PackageConstraint {
					package,
					version_constraint,
				};
			map.insert(package.package.name.clone(), package);
		}
	}
	map

}
