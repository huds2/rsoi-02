mod repository;
mod server;
use repository::*;
use structs::*;
use server::*;

use std::error::Error;
use std::env;
use custom_error::custom_error;

custom_error!{pub FlightError
    NotFoundError                            = "Ticket was not found",
}

#[macro_export]
macro_rules! arc{
    ($a:expr)=>{
        {
            std::sync::Arc::new(tokio::sync::Mutex::new($a))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    // let connection_str = env::var("PSQL_CONNECTION")?;
    // let port = env::var("SERVER_PORT")?.parse()?;
    let connection_str = "postgresql://program:test@localhost/flights";
    let port = 8060;
    let repository = arc!(Repository::new(&connection_str).await?);
    repository.lock().await.init().await?;
    run_server(repository, port).await;
    // let tickets = repository.lock().await.list().await?;
    // println!("{:?}", tickets);
    // let ticket = repository.lock().await.get(uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8")).await?;
    // println!("{:?}", ticket);
    // repository.lock().await.create(TicketPost { flight_number: "fligh2".to_string(), price: 54132 }, "someone".to_string()).await?;
    // repository.lock().await.cancel(uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8")).await?;
    Ok(())

}
