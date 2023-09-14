use nju_schedule_ics::server::server;
use nju_schedule_ics::server::error::AppError;
use tokio;

#[tokio::main]
async fn main() -> Result<(),AppError> {
    server::start_server_from_commandline().await?;

    Ok(())
}
