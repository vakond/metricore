//! Simple high-load HTTP service to gather metrics.
//! Metrics are sent by numerous external senders.
//! The service stores all received metrics in a file store.
//! The service should be capable to handle up to 10k rps.

use actix_web::web::{self, Bytes, Data};
use actix_web::{App, HttpResponse, HttpServer, Responder};
use chrono::offset::Utc;
use chrono::DateTime;
use futures;
use serde_json::{Map, Result, Value};
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex; // maybe futures::lock::Mutex is better here
use std::thread;
use std::time::{Duration, SystemTime};

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let state = Data::new(State::new());
    let ticker = run_ticker(state.clone());
    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::put().to(handler))
    })
    .bind("localhost:3000")?
    .run();
    let (result, _) = futures::join!(server, ticker);
    result
}

/// Handles each request. We interested only in the body of request.
/// The body is deserialized and new event added to the queue.
async fn handler((body, state): (Bytes, Data<State>)) -> impl Responder {
    let map: Result<Map<String, Value>> = serde_json::from_slice(&body.to_vec());
    match map {
        Err(_) => return HttpResponse::BadRequest(),
        Ok(map) => {
            const EV: &str = "event";
            if !map.contains_key(EV) {
                return HttpResponse::BadRequest();
            }
            let info = &map[EV];
            if !state.into_inner().add(info.to_string()) {
                return HttpResponse::InternalServerError();
            }
        }
    }
    HttpResponse::Ok()
}

/// Starts asynchronous loop to save events in db periodically.
async fn run_ticker(state: Data<State>) -> io::Result<()> {
    const FILENAME: &str = "events.txt";
    while !state.done {
        thread::sleep(Duration::from_secs(5));
        let mut queue = state.queue.lock().unwrap();
        {
            //print!("O={}", queue.len());
            //io::stdout().flush().unwrap();
            let exists = PathBuf::from(FILENAME).exists();
            let file = OpenOptions::new()
                .create(!exists)
                .append(true)
                .open(FILENAME);
            if file.is_err() {
                eprintln!("Failed to open file {}", FILENAME);
                io::stderr().flush().unwrap();
                continue;
            }
            let mut file = file.unwrap();
            for e in queue.iter() {
                file.write(e.as_string().as_bytes()).unwrap();
                file.write("\n".as_bytes()).unwrap();
            }
        }
        queue.clear();
    }
    Ok(())
}

/// Represents shared state of the service.
struct State {
    done: bool,
    queue: Mutex<Vec<Event>>,
}

impl State {
    /// Creates new instance of State.
    fn new() -> Self {
        State {
            done: false,
            queue: Mutex::new(Vec::new()),
        }
    }

    /// Adds event to the queue.
    fn add(&self, info: String) -> bool {
        let lock = self.queue.lock();
        if lock.is_err() {
            return false;
        }
        let mut guard = lock.unwrap();
        //print!(".");
        guard.push(Event::new(info));
        true
    }
}

/// Represents event.
#[derive(Debug)]
struct Event {
    info: String,
    timestamp: SystemTime,
}

impl Event {
    fn new(info: String) -> Self {
        Event {
            info,
            timestamp: SystemTime::now(),
        }
    }

    fn as_string(&self) -> String {
        let dt: DateTime<Utc> = self.timestamp.into();
        format!("{} {}", dt.format("%d/%m/%Y %T"), self.info)
    }
}
