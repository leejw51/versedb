use clap::Parser;
use versedb::csv::CsvDatabase;
use versedb::database::Database;
use versedb::json::JsonDatabase;
use versedb::memory::MemoryDatabase;
#[cfg(not(target_arch = "wasm32"))]
use versedb::sled::SledDatabase;
#[cfg(not(target_arch = "wasm32"))]
use versedb::sqlite::SqliteDatabase;
use versedb::yaml::YamlDatabase;

#[cfg(not(target_arch = "wasm32"))]
use versedb::server::run_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server address in the format host:port
    #[arg(short, long, default_value = "127.0.0.1:8000")]
    address: String,

    #[arg(long, default_value = "csv", help = "csv,json,sqlite,yaml,sled,memory")]
    dbtype: String,

    #[arg(long, default_value = "data.csv")]
    dbpath: String,
}

#[tokio::main]
#[cfg(not(target_arch = "wasm32"))]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.dbtype.as_str() {
        "csv" => {
            let db = CsvDatabase::open(&args.dbpath).await?;
            run_server(&args.address, db).await?;
        }
        "json" => {
            let db = JsonDatabase::open(&args.dbpath).await?;
            run_server(&args.address, db).await?;
        }
        "sqlite" => {
            let db = SqliteDatabase::open(&args.dbpath).await?;
            run_server(&args.address, db).await?;
        }
        "yaml" => {
            let db = YamlDatabase::open(&args.dbpath).await?;
            run_server(&args.address, db).await?;
        }
        "sled" => {
            let db = SledDatabase::open(&args.dbpath).await?;
            run_server(&args.address, db).await?;
        }
        "memory" => {
            let db = MemoryDatabase::open(&args.dbpath).await?;
            run_server(&args.address, db).await?;
        }
        _ => {
            eprintln!("Unsupported database type: {}", args.dbtype);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
