use clap::Parser;
use super::db;

#[derive(Parser,Debug)]
#[command(author,version,about,long_about=None)]
struct Args{
    // Redis connection URL
    #[arg(short,long)]
    url: String,
}

pub async fn get_db() -> Result<db::RedisDb,anyhow::Error> {
    let args=Args::parse();

    let db=db::RedisDb::new(&args.url).await?;

    Ok(db)
}


