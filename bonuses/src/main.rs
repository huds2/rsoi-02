mod repository;
mod server;
use repository::*;
use server::*;

use std::error::Error;
use std::env;
use custom_error::custom_error;

custom_error!{pub PrivilegeError
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
    let connection_str = "postgresql://program:test@localhost/privileges";
    let port = 8050;
    let repository = arc!(Repository::new(&connection_str).await?);
    repository.lock().await.init().await?;
    run_server(repository, port).await;
    // let privilege = repository.lock().await.get_privilege("me".to_string()).await?;
    // println!("{:?}", privilege);
    // let history = repository.lock().await.get_privilege_history("me".to_string()).await?;
    // println!("{:?}", history);
    // let history = repository.lock().await.get_privilege_history_by_ticket(uuid::uuid!("077d2570-0612-4767-a519-73bf973ed689")).await?;
    // println!("{:?}", history);
    // let ticket = repository.lock().await.get(uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8")).await?;
    // println!("{:?}", ticket);
    // repository.lock().await.create(TicketPost { flight_number: "fligh2".to_string(), price: 54132 }, "someone".to_string()).await?;
    // repository.lock().await.cancel(uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8")).await?;
    Ok(())

}
