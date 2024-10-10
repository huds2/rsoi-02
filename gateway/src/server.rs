use std::{collections::HashMap, convert::Infallible, sync::Arc, error::Error};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{reply::{self, Reply}, Filter, Rejection};
use requester::{send_typed, RequestMethod, Requester};
use structs::{Balance, CombinedPurchaseResponse, PrivilegeGet, PurchasePost, PurchaseResponse, Ticket, TicketPost, TicketPostBalance, TicketResponse, User, WebFlight, WebFlightPage};
use crate::GatewayError;

pub type WebResult<T> = std::result::Result<T, Rejection>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    pub page: Option<usize>,
    pub size: Option<usize>,
}

async fn list_flights_handler(paging: Paging,
                            services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let flight_url = services.lock().await.flights.clone();
    let requester = &services.lock().await.requester.clone();
    let Ok(flights) = send_typed::<WebFlightPage>(
        &requester,
        format!("{}?page={}&size={}", flight_url, paging.page.unwrap_or(1), paging.size.unwrap_or(10)),
        RequestMethod::GET,
        HashMap::new(),
        "".to_string()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    return Ok(Box::new(reply::json(&flights)));
}

async fn ticket_to_responseticket(ticket: Ticket, 
                                  services: Arc<Mutex<Services>>) -> Result<TicketResponse, Box<dyn Error>> {
    let flights_url = services.lock().await.flights.clone();
    let requester = &services.lock().await.requester.clone();
    let Ok(flight) = send_typed::<WebFlight>(
        requester,
        format!("{}/{}", flights_url, ticket.flight_number),
        RequestMethod::GET,
        HashMap::new(),
        "".to_owned()).await
    else {
        return Err(Box::new(GatewayError::FlightNotFoundError));
    };
    Ok(TicketResponse {
        date: flight.date,
        ticketUid: ticket.ticket_uid,
        flightNumber: flight.flightNumber,
        fromAirport: flight.fromAirport,
        toAirport: flight.toAirport,
        price: flight.price,
        status: ticket.status
    })
}

async fn list_tickets_handler(username: String,
                              services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let ticket_url = services.lock().await.tickets.clone();
    let requester = &services.lock().await.requester.clone();
    let Ok(tickets) = send_typed::<Vec<Ticket>>(
        &requester,
        ticket_url,
        RequestMethod::GET,
        HashMap::from([
            ("X-User-Name".to_owned(), username)
        ]),
        "".to_string()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let mut response_tickets = vec![];
    for ticket in tickets {
        let Ok(ticket) = ticket_to_responseticket(ticket, services.clone()).await else {
            let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
            return Ok(Box::new(reply));
        };
        response_tickets.push(ticket);
    }
    return Ok(Box::new(reply::json(&response_tickets)));
}

async fn get_ticket_handler(ticket_uid: Uuid,
                            username: String,
                            services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let ticket_url = services.lock().await.tickets.clone();
    let requester = &services.lock().await.requester.clone();
    let Ok(ticket) = send_typed::<Ticket>(
        &requester,
        format!("{}/{}", ticket_url, ticket_uid),
        RequestMethod::GET,
        HashMap::from([
            ("X-User-Name".to_owned(), username)
        ]),
        "".to_string()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let Ok(ticket) = ticket_to_responseticket(ticket, services.clone()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    return Ok(Box::new(reply::json(&ticket)));
}

async fn get_privilege_handler(username: String,
                               services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let privilege_url = services.lock().await.bonuses.clone();
    let requester = &services.lock().await.requester.clone();
    let Ok(privilege) = send_typed::<PrivilegeGet>(
        &requester,
        privilege_url,
        RequestMethod::GET,
        HashMap::from([
            ("X-User-Name".to_owned(), username)
        ]),
        "".to_string()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    return Ok(Box::new(reply::json(&privilege)));
}

async fn get_user_handler(username: String,
                               services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let privilege_url = services.lock().await.bonuses.clone();
    let ticket_url = services.lock().await.tickets.clone();
    let requester = &services.lock().await.requester.clone();
    let Ok(privilege) = send_typed::<PrivilegeGet>(
        &requester,
        privilege_url,
        RequestMethod::GET,
        HashMap::from([
            ("X-User-Name".to_owned(), username.clone())
        ]),
        "".to_string()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let Ok(tickets) = send_typed::<Vec<Ticket>>(
        &requester,
        ticket_url,
        RequestMethod::GET,
        HashMap::from([
            ("X-User-Name".to_owned(), username)
        ]),
        "".to_string()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let mut response_tickets = vec![];
    for ticket in tickets {
        let Ok(ticket) = ticket_to_responseticket(ticket, services.clone()).await else {
            let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
            return Ok(Box::new(reply));
        };
        response_tickets.push(ticket);
    }
    return Ok(Box::new(reply::json(&User{
        tickets: response_tickets,
        privilege: Balance {
            balance: privilege.balance,
            status: privilege.status
        }
    })));
}

async fn post_ticket_handler(username: String,
                             body: TicketPostBalance,
                             services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let ticket_url = services.lock().await.tickets.clone();
    let privilege_url = services.lock().await.bonuses.clone();
    let requester = &services.lock().await.requester.clone();
    let ticket_post = TicketPost {
        flight_number: body.flightNumber,
        price: body.price
    };
    let Ok(ticket) = send_typed::<Ticket>(
        requester,
        ticket_url,
        RequestMethod::POST,
        HashMap::from([
            ("X-User-Name".to_owned(), username.clone())
        ]),
        serde_json::to_string(&ticket_post).unwrap()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let privilege_post = PurchasePost {
        ticket_uid: ticket.ticket_uid,
        price: ticket.price,
        paid_from_balance: body.paidFromBalance
    };
    let Ok(purchase) = send_typed::<PurchaseResponse>(
        requester,
        privilege_url,
        RequestMethod::POST,
        HashMap::from([
            ("X-User-Name".to_owned(), username)
        ]),
        serde_json::to_string(&privilege_post).unwrap()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let Ok(ticket) = ticket_to_responseticket(ticket, services.clone()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    return Ok(Box::new(reply::json(&CombinedPurchaseResponse {
        ticketUid: ticket.ticketUid,
        flightNumber: ticket.flightNumber,
        fromAirport: ticket.fromAirport,
        toAirport: ticket.toAirport,
        date: ticket.date,
        price: ticket.price,
        paidByMoney: purchase.paid_by_money,
        paidByBonuses: purchase.paid_by_bonuses,
        status: ticket.status,
        privilege: Balance { 
            balance: purchase.balance,
            status: purchase.status 
        }
    })));
}

async fn delete_ticket_handler(ticket_uid: Uuid, 
                               username: String,
                               services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let ticket_url = services.lock().await.tickets.clone();
    let privilege_url = services.lock().await.bonuses.clone();
    let requester = &services.lock().await.requester.clone();
    let Ok(ticket) = send_typed::<Ticket>(
        &requester,
        format!("{}/{}", ticket_url, ticket_uid),
        RequestMethod::GET,
        HashMap::from([
            ("X-User-Name".to_owned(), username.clone())
        ]),
        "".to_string()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    if ticket.status != "PAID" {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::BAD_REQUEST);
        return Ok(Box::new(reply));
    }
    let Ok(response) = requester.send(
        format!("{}/{}", ticket_url, ticket_uid),
        RequestMethod::DELETE,
        HashMap::from([
            ("X-User-Name".to_owned(), username.clone())
        ]),
        "".to_owned()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    if response.code != 204 {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    }
    let Ok(response) = requester.send(
        format!("{}?ticket_uid={}", privilege_url, ticket_uid),
        RequestMethod::DELETE,
        HashMap::from([
            ("X-User-Name".to_owned(), username)
        ]),
        "".to_owned()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    if response.code != 204 {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    }
    let reply = warp::reply::with_status("", warp::http::StatusCode::NO_CONTENT);
    return Ok(Box::new(reply));
}

async fn health_check_handler() -> WebResult<impl Reply> {
    return Ok(warp::reply::with_status("Up and running", warp::http::StatusCode::OK))
}

fn with_arc<T: Send + ?Sized>(arc: Arc<Mutex<T>>) -> impl Filter<Extract = (Arc<Mutex<T>>,), Error = Infallible> + Clone {
    warp::any().map(move || arc.clone())
}

#[derive(Clone)]
pub struct Services {
    pub flights: String,
    pub tickets: String,
    pub bonuses: String,
    pub requester: Box<dyn Requester>
}

pub fn router(root_url: &str, services: Arc<Mutex<Services>>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let list_flights_route = warp::path!("flights")
        .and(warp::get())
        .and(warp::query::<Paging>())
        .and(with_arc(services.clone()))
        .and_then(list_flights_handler);
    let list_tickets_route = warp::path!("tickets")
        .and(warp::get())
        .and(warp::header("X-User-Name"))
        .and(with_arc(services.clone()))
        .and_then(list_tickets_handler);
    let get_ticket_route = warp::path!("tickets" / Uuid)
        .and(warp::get())
        .and(warp::header("X-User-Name"))
        .and(with_arc(services.clone()))
        .and_then(get_ticket_handler);
    let get_privilege_route = warp::path!("privilege")
        .and(warp::get())
        .and(warp::header("X-User-Name"))
        .and(with_arc(services.clone()))
        .and_then(get_privilege_handler);
    let get_user_route = warp::path!("me")
        .and(warp::get())
        .and(warp::header("X-User-Name"))
        .and(with_arc(services.clone()))
        .and_then(get_user_handler);
    let post_ticket_route = warp::path!("tickets")
        .and(warp::post())
        .and(warp::header("X-User-Name"))
        .and(warp::body::json())
        .and(with_arc(services.clone()))
        .and_then(post_ticket_handler);
    let delete_ticket_route = warp::path!("tickets"/ Uuid)
        .and(warp::delete())
        .and(warp::header("X-User-Name"))
        .and(with_arc(services.clone()))
        .and_then(delete_ticket_handler);
    let health_route = warp::path!("manage" / "health")
        .and(warp::get())
        .and_then(health_check_handler);
    let routes = list_flights_route
        .or(list_tickets_route)
        .or(get_ticket_route)
        .or(get_privilege_route)
        .or(get_user_route)
        .or(post_ticket_route)
        .or(delete_ticket_route)
        .or(health_route);
    let mut root_route = warp::any().boxed();
    for segment in root_url.split("/") {
        root_route = root_route.and(warp::path(segment.to_owned())).boxed();
    }
    let routes = root_route.and(routes);
    routes
}

pub async fn run_server(root_url: &str, port: u16, services: Arc<Mutex<Services>>) {
    let router = router(root_url, services);
    warp::serve(router)
        .run(([0, 0, 0, 0], port))
        .await
}
