use regex::Regex;
use std::path::{Path, PathBuf};

pub fn parse_lfs_output(output: &str) -> Vec<PathBuf> {
    let re = Regex::new(r"^([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]+)").expect("valid regex");

    output
        .lines()
        .filter_map(|line| {
            re.captures(line).map(|caps| {
                let p1 = &caps[1];
                let p2 = &caps[2];
                let rest = &caps[3];

                Path::new(".git")
                    .join("lfs")
                    .join("objects")
                    .join(p1)
                    .join(p2)
                    .join(format!("{p1}{p2}{rest}"))
            })
        })
        .collect()
}
