use std::{sync::RwLock};
// use sha2::Sha256;
// use sha2::Digest;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub struct Barrel {
    items: RwLock<Vec<u64>>
}

fn get_key(id: u32, project: &str, repository: &str, branch: &str) -> u64 {
    let key = format!("{}-{}-{}-{}", id, project, repository, branch);
    let mut s = DefaultHasher::new();
    s.write(&key.as_bytes());
    s.finish()
    // let mut hasher = Sha256::new();
    // hasher.update(key.as_bytes());
    // let result = hasher.finalize();
    // format!("{:x}", result)
}

impl Barrel {
    const fn new() -> Barrel {
        Barrel { 
            items: RwLock::new(vec![])
        }
    }

    pub fn try_put(&self, id: u32, project: &str, repository: &str, branch: &str) -> Result<(), &'static str>{
        let key = get_key(id, project, repository, branch);
        let found = self.find(key);
        if found {
            return Err("duplicated");
        }
        self.add(key);
        Ok(())
    }

    fn find(&self, key: u64) -> bool {
        let item = self.items.read().unwrap();
        item.iter().any(|x| *x == key)
    }

    fn add(&self, key: u64) {
        let mut item = self.items.write().unwrap();
        item.push(key);
    }

    pub fn remove(&self, id: u32, project: &str, repository: &str, branch: &str) {
        let key = get_key(id, project, repository, branch);
        let mut item = self.items.write().unwrap();
        let index = item.iter().position(|x| *x == key);
        if let Some(idx) = index {
            item.remove(idx);
        }
        return;
    }
}
pub const GLOBAL_BARREL: Barrel = Barrel::new();

pub async fn run_command(username: &str,
                         password: &str,
                         id: u32,
                         project_to: &str,
                         repository_to: &str,
                         branch_to: &str,
                         commit_to: &str,
                         project_from: &str,
                         repository_from: &str,
                         branch_from: &str,
                         commit_from: &str,) {
    let mut command = tokio::process::Command::new("ls");
    command
        .env("USERNAME", username)
        .env("PASSWORD", password);
    let mut proc = command.spawn().unwrap();
    let ret = proc.wait().await;
    match ret {
        Ok(_) => tracing::trace!("program ran ok"),
        Err(e) => tracing::info!("program ran with error {:?}", e),
    }
    GLOBAL_BARREL.remove(id, project_to, repository_to, branch_to);
}