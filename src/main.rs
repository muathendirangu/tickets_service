use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{
    delete, get, post, put, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
    ResponseError,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    id: u32,
    author: String,
}

// Implement Responder Trait for Ticket
impl Responder for Ticket {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let res_body = serde_json::to_string(&self).unwrap();
        // Create HttpResponse and set Content Type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(res_body)
    }
}

#[derive(Debug, Serialize)]
struct ErrNoId {
    id: u32,
    err: String,
}

// Implement ResponseError for ErrNoId
impl ResponseError for ErrNoId {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let body = serde_json::to_string(&self).unwrap();
        let res = HttpResponse::new(self.status_code());
        res.set_body(BoxBody::new(body))
    }
}

// Implement Display for ErrNoId
impl Display for ErrNoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

//Define AppState
struct AppState {
    tickets: Mutex<Vec<Ticket>>,
}

//handlers
// Create a ticket
#[post("/tickets")]
async fn create_ticket_handler(
    req: web::Json<Ticket>,
    data: web::Data<AppState>,
) -> impl Responder {
    let new_ticket = Ticket {
        id: req.id,
        author: String::from(&req.author),
    };

    let mut tickets = data.tickets.lock().unwrap();

    let response = serde_json::to_string(&new_ticket).unwrap();

    tickets.push(new_ticket);

    HttpResponse::Created()
        .content_type(ContentType::json())
        .body(response)
}

// Get all tickets
#[get("/tickets")]
async fn get_tickets_handler(data: web::Data<AppState>) -> impl Responder {
    let tickets = data.tickets.lock().unwrap();

    let response = serde_json::to_string(&(*tickets)).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response)
}

// Get a ticket given an id
#[get("/tickets/{id}")]
async fn get_ticket_handler(
    id: web::Path<u32>,
    data: web::Data<AppState>,
) -> Result<Ticket, ErrNoId> {
    let ticket_id = *id;
    let tickets = data.tickets.lock().unwrap();
    let ticket: Vec<_> = tickets.iter().filter(|t| t.id == ticket_id).collect();
    if !ticket.is_empty() {
        let ticket = Ticket {
            id: ticket[0].id,
            author: String::from(&ticket[0].author),
        };
        Ok(ticket)
    } else {
        let response = ErrNoId {
            id: ticket_id,
            err: String::from("ticket not found"),
        };
        Err(response)
    }
}

// Update the ticket given an id
#[put("/tickets/{id}")]
async fn update_ticket_handler(
    id: web::Path<u32>,
    req: web::Json<Ticket>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, ErrNoId> {
    let ticket_id: u32 = *id;

    let new_ticket = Ticket {
        id: req.id,
        author: String::from(&req.author),
    };

    let mut tickets = data.tickets.lock().unwrap();
    let id_index = tickets.iter().position(|x| x.id == ticket_id);
    match id_index {
        Some(id) => {
            let response = serde_json::to_string(&new_ticket).unwrap();
            tickets[id] = new_ticket;
            Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response))
        }
        None => {
            let response = ErrNoId {
                id: ticket_id,
                err: String::from("no ticket found"),
            };
            Err(response)
        }
    }
}
// Delete a ticket given an id
#[delete("/tickets/{id}")]
async fn delete_ticket_handler(
    id: web::Path<u32>,
    data: web::Data<AppState>,
) -> Result<Ticket, ErrNoId> {
    let ticket_id: u32 = *id;
    let mut tickets = data.tickets.lock().unwrap();

    let id_index = tickets.iter().position(|x| x.id == ticket_id);

    match id_index {
        Some(id) => {
            let deleted_ticket = tickets.remove(id);
            Ok(deleted_ticket)
        }
        None => {
            let response = ErrNoId {
                id: ticket_id,
                err: String::from("no ticket  found"),
            };
            Err(response)
        }
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        tickets: Mutex::new(vec![
            Ticket {
                id: 1,
                author: String::from("Charles"),
            },
            Ticket {
                id: 2,
                author: String::from("Brad"),
            },
        ]),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(create_ticket_handler)
            .service(get_tickets_handler)
            .service(delete_ticket_handler)
            .service(get_ticket_handler)
            .service(update_ticket_handler)
    })
    .bind("127.0.0.1:4356")?
    .run()
    .await
}
