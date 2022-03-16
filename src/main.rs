use std::{env, fs, path::Path, time::Duration};
// use swc;
use tokio::time::sleep;

use crate::swc::swc_main;

mod executor;
mod swc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let path = fs::canonicalize(path).unwrap();
    println!("{:?}", path);
    // let path = Path::new("output/test.ts");
    let source = tokio::fs::read(&path).await?;
    let code = swc_main(&String::from_utf8_lossy(&source).to_string(), &path);
    tokio::spawn(sleep(Duration::from_micros(1)));

    executor::run(code);

    Ok(())
}
