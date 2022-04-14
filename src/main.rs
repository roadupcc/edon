use std::{env, fs, path::Path};

use crate::swc::swc_main;

mod runner;
mod runtime;
mod swc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let path = fs::canonicalize(path).unwrap();
    println!("{:?}", path);

    let source = tokio::fs::read(&path).await?;
    let code = swc_main(&String::from_utf8_lossy(&source).to_string(), &path);

    let mut rtm = runtime::JTsRuntime::new();

    let r = rtm.mod_evaluate(code);
    let result = r.await?;
    println!("{:#?}", result);

    Ok(())
}
