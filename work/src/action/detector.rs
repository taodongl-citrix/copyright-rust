use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use rayon::prelude::*;
use regex::Regex;
use crate::action::{COPYRIGHT_CSHARP, COPYRIGHT_JS};

pub struct Detector {
    filters : Vec<glob::Pattern>,
    workspace: PathBuf,
}

impl Detector {
    pub fn new() -> Detector {
        let workspace = std::env::current_dir().expect("cannot get work dir");
        let filters : Vec<glob::Pattern> = vec![];
        Detector {
            workspace,
            filters
        }
    }

    pub fn scan(&self) -> anyhow::Result<bool> {
        let mut files: Vec<PathBuf> = vec![];
        for entry in walkdir::WalkDir::new(&self.workspace)
            .into_iter()
            .filter_entry(|e| !self.is_hidden(e))
            .filter_map(|e| e.ok()) {
            if entry.metadata().unwrap().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
        let number:i32 = files.par_iter().map(parse).sum();
        Ok(number > 0)
    }

    fn is_hidden(&self, entry: &walkdir::DirEntry) -> bool {
        let yes = entry.file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false);
        if yes {
            return true;
        }
        let pos = self.filters.iter().position(|x| x.matches_path(entry.path()));
        match pos {
            Some(_x) => return true,
            None => false
        }
    }
}

fn parse(path: &PathBuf) -> i32 {
    let ext = path.extension().unwrap();
    if ext == "cs" {
        return parse_file_with_language(&path, COPYRIGHT_CSHARP);
    } else if ext == "js" {
        return parse_file_with_language(&path, COPYRIGHT_JS);
    }
    0
}

fn parse_file_with_language(file: &PathBuf, copyright: &str) -> i32 {
    let reg_str = format!("^{}", copyright.replace("\n", r"[ \r\n]+"));
    let pattern = Regex::new(&reg_str).unwrap();
    match parse_file(file, &pattern) {
        Ok(found) => {
            if found {
                return 1;
            }
        },
        Err(err) => tracing::error!("{}", err)
    }
    return 0;
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
        None => encoding_rs::UTF_8.decode_with_bom_removal(&bytes)
    };
    if text.1 {
        return Err(anyhow::anyhow!("fail to decode file content"))
    }
    let found = pattern.is_match(&text.0);
    if found {
        return Ok(false);
    }
    Ok(true)
}