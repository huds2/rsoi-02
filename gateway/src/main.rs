use std::error::Error;
use custom_error::custom_error;

use requester::Reqwester;
mod server;
use server::*;

custom_error!{pub GatewayError
    TicketNotFoundError                             = "Ticket was not found",
    FlightNotFoundError                             = "Flight was not found",
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
    // let port = env::var("SERVER_PORT")?.parse()?;
    let port = 8080;
    run_server("api/v1", port, arc!(Services { 
        flights: "http://localhost:8060/flights".to_owned(),
        tickets: "http://localhost:8070/tickets".to_owned(),
        bonuses: "http://localhost:8050/privilege".to_owned(),
        requester: Box::new(Reqwester {})
    })).await;
    Ok(())
}
