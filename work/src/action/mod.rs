pub mod bitbucket;
mod detector;
pub mod github;
mod models;

const SUPPORT_FILE_TYPES: &'static [&str] = &["cs", "java", "ts", "js"];

pub trait Handler {
    fn execute(&mut self, project: &str, repository: &str, id: u32) -> anyhow::Result<()>;
}

pub use bitbucket::Bitbucket;
pub use github::Github;

pub fn run_command(args: &[&str]) -> anyhow::Result<()> {
    println!("> git {}", args.join(" "));
    std::process::Command::new("git").args(args).status()?;
    Ok(())
}

pub fn git_fetch(files: &[String], url: &str, id: u32) -> anyhow::Result<()> {
    run_command(&["remote", "add", "origin", url])?;
    run_command(&[
        "sparse-checkout",
        "set",
        "--no-cone",
        "pipeline.yaml",
        "pipeline.yml",
    ])?;
    for file in files.iter() {
        let ext = std::path::Path::new(file)
            .extension()
            .and_then(std::ffi::OsStr::to_str);
        if let Some(e) = ext {
            if SUPPORT_FILE_TYPES.contains(&e) {
                run_command(&["sparse-checkout", "add", file])?;
            }
        }
    }
    run_command(&[
        "fetch",
        "--no-tags",
        "--depth=1",
        "origin",
        &format!("+refs/pull/{id}/head:refs/remotes/origin/PR-{id}", id = id),
    ])?;
    run_command(&["checkout", &format!("PR-{id}", id = id)])?;
    Ok(())
}

pub static COPYRIGHT_CSHARP: &'static str = include_str!("copyright_csharp.txt");
pub static COPYRIGHT_JS: &'static str = include_str!("copyright_js.txt");

pub fn scan() -> anyhow::Result<bool> {
    let detector = detector::Detector::new();
    detector.scan()
}
