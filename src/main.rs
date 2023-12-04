#[macro_use]
extern crate rocket;
use std::collections::HashMap;
use std::net::SocketAddr;
mod entities;
mod fairings;
mod manage_requests;

use crate::entities::storedrequest::StoredRequest;
use crate::fairings::de_construct_request::RRequest;
use crate::manage_requests::request_funcs::{
    delete_request_from_db, send_stored_request, write_request_to_db,
};
use config::Config;
use log::info;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config as LogConfig;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::tokio::time::{interval_at, Instant};
use rocket::Response;
use rocket::{custom, tokio};
use sqlx::{MySqlPool};

// // // // // // // // // // // // // // // // // // // // // // // //
// // // // // // // // // // // // // // // // // // // // // // // //

// https://rocket.rs/v0.5/overview
#[get("/")]
async fn index(socket_addr: SocketAddr, pool: &rocket::State<MySqlPool>) -> &'static str {
    let is_pool_closed = pool.is_closed();
    info!(target:"app::requests", "ROOT PATH - From: {}", socket_addr.ip().to_string());
    if is_pool_closed {
        "No Swimming"
    } else {
        "Hello, Astronauts!"
    }
}

#[post("/your/endpoint", data = "<data>")]
async fn your_endpoint(
    request: RRequest,
    pool: &rocket::State<MySqlPool>, // see line 237, wrapping this in a State<> signals Rocket to bring this into scope
    data: String,
    _settings: &rocket::State<HashMap<String, String>>,
) -> Result<(), ErrorResponder> {
    println!("{:?}", request);

    let new_req = StoredRequest { // src/entities/storedrequest.rs
        method: request.method,
        host: request.host,
        port: 80,
        uri: request.uri,
        headers: request.headers,
        body: data,
    };

    let _ = write_request_to_db(new_req.clone(), pool).await; // create stored request and insert into database, ignoring the result

    Ok(())
}

#[post("/delay/<delay_num>", data = "<data>")]
async fn slow_test_server( // for testing purposes, https://github.com/helloimalemur/Slow-Server to simulate slow-responding server
    delay_num: i64,
    request: RRequest,
    pool: &rocket::State<MySqlPool>,
    data: String,
) -> Result<(), ErrorResponder> {
    println!("{:?} \n--- delay: {}", request, delay_num);

    let new_req = StoredRequest {
        method: request.method,
        host: request.host,
        port: 80,
        uri: request.uri,
        headers: request.headers,
        body: data,
    };

    let _ = write_request_to_db(new_req.clone(), pool).await;

    Ok(())
}

// // // // // // // // // // // // // // // // // // // // // // // //
// // // // // // // // // // // // // // // // // // // // // // // //

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_status(Status::new(200));
    }
}

// // // // // // // // // // // // // // // // // // // // // // // //
// // // // // // // // // // // // // // // // // // // // // // // //

#[rocket::main]
pub async fn main() {
    // load configuration file
    let settings = Config::builder()
        .add_source(config::File::with_name("config/Settings"))
        .build()
        .unwrap();
    let settings_map = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let config = rocket::Config {
        port: 8030,
        address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
        ..rocket::Config::debug_default()
    };

    // setup logging request logging to file
    let stdout = ConsoleAppender::builder().build();
    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(settings_map.get("log_path").unwrap().as_str())
        .unwrap();
    #[allow(unused_variables)]
    let log_config = LogConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("requests", Box::new(requests)))
        // .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(
            Logger::builder()
                .appender("requests")
                .additive(true)
                .build("app::requests", LevelFilter::Info),
        )
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    // logging to info
    info!(target: "app::requests","Starting");

    // set database_url string
    let database_url: &str = settings_map.get("database_url").unwrap().as_str();

    println!("{}", database_url);

    let interval_pool = MySqlPool::connect(database_url)
        .await
        .expect("database connection");

    // start re-occuring task to send requests slowly
    let http_dest: String = settings_map.get("http_dest").unwrap().to_string();
    let http_proto: String = settings_map.get("http_proto").unwrap().to_string();
    let reque_interval: u64 = settings_map
        .get("reque_interval")
        .unwrap()
        .to_string()
        .parse::<u64>()
        .unwrap();
    tokio::spawn(async move {
        let start = Instant::now();
        let mut interval = interval_at(start, tokio::time::Duration::from_secs(reque_interval));

        loop {
            interval.tick().await;
            let out = sqlx::query("SELECT * FROM requests ORDER BY id ASC")
                .fetch_one(&interval_pool.clone())
                .await;

            // let mut method: String = String::new(); // filter incoming by method in the future?
            // let mut host: String = String::new(); // filter by host in the future?
            let uri: String = String::new();
            let body: String = String::new();

            let out_ok = out.is_ok();
            if out_ok {

                // println!("{} - {}", uri, body);
                let send_success = send_stored_request(
                    http_proto.clone(),
                    http_dest.clone(),
                    uri.clone(),
                    body.clone(),
                )
                .await;
                if send_success {
                    delete_request_from_db(uri.clone(), body.clone(), &interval_pool).await;
                }
            }
        }
    });

    // initialize database connection
    let pool = MySqlPool::connect(database_url)
        .await
        .expect("database connection");

    // launch Rocket
    custom(&config)
        .manage(settings_map.clone())
        .manage::<MySqlPool>(pool)
        .mount("/", routes![index, your_endpoint, slow_test_server])
        .attach(CORS)
        .launch()
        .await
        .unwrap();
}

// The following impl's are for easy conversion of error types.
#[derive(Responder)]
#[response(status = 500, content_type = "json")]
struct ErrorResponder {
    message: String,
}

impl From<anyhow::Error> for ErrorResponder {
    fn from(err: anyhow::Error) -> ErrorResponder {
        ErrorResponder {
            message: err.to_string(),
        }
    }
}

impl From<String> for ErrorResponder {
    fn from(string: String) -> ErrorResponder {
        ErrorResponder { message: string }
    }
}

impl From<&str> for ErrorResponder {
    fn from(str: &str) -> ErrorResponder {
        str.to_owned().into()
    }
}
