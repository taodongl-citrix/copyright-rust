mod action;

use crate::action::{Bitbucket, Github, Handler};
use clap::{Parser, ValueEnum};
use rand::distributions::{Alphanumeric, DistString};
use std::path::{Path, PathBuf};
use tracing_subscriber::filter::FilterExt;
use tracing_subscriber::filter::{filter_fn, LevelFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    project: String,
    #[arg(short, long)]
    repository: String,
    #[arg(short, long)]
    id: u32,
    #[arg(short, long, value_enum)]
    scm: ScmType,
}
#[derive(ValueEnum, Clone)]
enum ScmType {
    Github,
    Bitbucket,
}

struct Workspace(PathBuf);

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
        tracing::info!("clean directory when quit!!!")
    }
}

fn main() -> anyhow::Result<()> {
    let target_filter = filter_fn(|meta| meta.target().starts_with("work"));
    let level_filter = LevelFilter::TRACE;
    let filter = target_filter.and(level_filter);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();
    let args: Args = Args::parse();
    let span = tracing::span!(
        tracing::Level::TRACE,
        "copyright",
        space = format!("{}:{}:{}", &args.project, &args.repository, args.id)
    );
    let _span_handle = span.enter();
    let handler: Option<Box<dyn Handler>> = match args.scm {
        ScmType::Github => Some(Box::new(Github::new(
            &args.project,
            &args.repository,
            args.id,
        ))),
        ScmType::Bitbucket => Some(Box::new(Bitbucket::new(
            &args.project,
            &args.repository,
            args.id,
        ))),
    };
    match handler {
        Some(mut h) => {
            let workspace = create_workspace(&args.project, &args.repository, args.id);
            std::process::Command::new("git")
                .arg("init")
                .arg(&workspace.0.to_str().unwrap())
                .output()
                .expect("failed to execute 'git init'");
            std::env::set_current_dir(&workspace.0).expect(&workspace.0.to_str().unwrap());
            tracing::info!("start in {}", &workspace.0.as_os_str().to_str().unwrap());
            h.execute()
        }
        None => Err(anyhow::anyhow!("nothing")),
    }
}

fn create_workspace(project: &str, repository: &str, id: u32) -> Workspace {
    let folder_name = format!(
        "{}_{}-{}_{}",
        project,
        repository,
        id,
        Alphanumeric.sample_string(&mut rand::thread_rng(), 16),
    );
    let workspace = std::env::current_dir().expect("cannot get work dir");
    let root = Path::new(&workspace);
    let work_dir = root.join(&folder_name);
    Workspace(work_dir)
}
