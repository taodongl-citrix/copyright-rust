use crate::action::{COPYRIGHT_CSHARP, COPYRIGHT_JS};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Filter {
    copyright: Vec<String>,
}

pub struct Detector {
    copyright_matches: HashMap<&'static str, Regex>,
    filters: Vec<glob::Pattern>,
    workspace: PathBuf,
}

impl Detector {
    pub fn new() -> Detector {
        let mut filters: Vec<glob::Pattern> = vec![];
        let workspace = std::env::current_dir().expect("cannot get work dir");
        let mut conf = workspace.join("pipeline.yaml");
        if !conf.is_file() {
            conf = workspace.join("pipeline.yml");
        }
        if conf.is_file() {
            let data: Result<Filter, anyhow::Error> = std::fs::read_to_string(conf)
                .map_err(|e| anyhow::anyhow!(e.to_string()))
                .and_then(|x| serde_yaml::from_str(&x).map_err(|e| anyhow::anyhow!(e.to_string())));
            if let Ok(o) = data {
                for f in o.copyright.iter() {
                    let pattern = glob::Pattern::new(f);
                    if let Ok(p) = pattern {
                        filters.push(p);
                    }
                }
            }
        }
        let copyright_matches = HashMap::from([
            ("cs", create_copyright_reg(COPYRIGHT_CSHARP)),
            ("js", create_copyright_reg(COPYRIGHT_JS)),
        ]);
        Detector {
            workspace,
            filters,
            copyright_matches,
        }
    }

    pub fn scan(&self) -> anyhow::Result<bool> {
        let mut files: Vec<PathBuf> = vec![];
        for entry in walkdir::WalkDir::new(&self.workspace)
            .into_iter()
            .filter_entry(|e| !self.is_hidden(e))
            .filter_map(|e| e.ok())
        {
            if entry.metadata().unwrap().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
        let number: i32 = files.par_iter().map(|x| self.parse(x)).sum();
        Ok(number > 0)
    }

    fn is_hidden(&self, entry: &walkdir::DirEntry) -> bool {
        let yes = entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false);
        if yes {
            return true;
        }
        let related_path = match entry.path().strip_prefix(&self.workspace) {
            Ok(p) => p.to_str().unwrap().replace("\\", "/"),
            Err(_x) => "".to_string(),
        };
        let pos = self.filters.iter().position(|x| {
            x.matches_with(
                &related_path,
                glob::MatchOptions {
                    case_sensitive: false,
                    require_literal_separator: true,
                    require_literal_leading_dot: true,
                },
            )
        });
        match pos {
            Some(_x) => return true,
            None => false,
        }
    }
    fn parse(&self, path: &PathBuf) -> i32 {
        let ext = path.extension().unwrap().to_str().unwrap();
        if ["cs", "js"].contains(&ext) {
            let ret = parse_file(&path, self.copyright_matches.get(ext).unwrap());
            match ret {
                Ok(found) => {
                    if found {
                        return 1;
                    }
                }
                Err(err) => tracing::error!("{}", err),
            }
        }
        0
    }
}

fn parse_file(file: &PathBuf, pattern: &Regex) -> anyhow::Result<bool> {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(file)?;
    // read file
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)?;
    let encoding = encoding_rs::Encoding::for_bom(&bytes);
    let text = match encoding {
        Some(x) => x.0.decode_with_bom_removal(&bytes),
        None => encoding_rs::UTF_8.decode_with_bom_removal(&bytes),
    };
    if text.1 {
        return Err(anyhow::anyhow!("fail to decode file content"));
    }
    let found = pattern.is_match(&text.0);
    if found {
        return Ok(false);
    }
    Ok(true)
}

fn create_copyright_reg(content: &str) -> Regex {
    let reg_str = content
        .lines()
        .into_iter()
        .map(|line| format!("{}\\s*\\n", line))
        .collect::<Vec<String>>()
        .join("\n");
    Regex::new(&reg_str).unwrap()
}
