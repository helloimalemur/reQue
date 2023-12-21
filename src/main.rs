#[macro_use]
extern crate rocket;
use chrono::Local;
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
use log4rs::config::{Appender, Root};
use log4rs::Config as LogConfig;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::tokio::time::{interval_at, Instant};
use rocket::Response;
use rocket::{custom, tokio};
use sqlx::{MySqlPool, Row};

// // // // // // // // // // // // // // // // // // // // // // // //
// // // // // // // // // // // // // // // // // // // // // // // //

// https://rocket.rs/v0.5/overview
#[get("/")]
async fn index<'a>(
    request: RRequest<'a>,
    socket_addr: SocketAddr,
    pool: &rocket::State<MySqlPool>,
) -> Result<(), ErrorResponder> {
    let _is_pool_closed = pool.is_closed();
    info!(target:"app::requests", "ROOT PATH - From: {}", socket_addr.ip().to_string());
    let now = Local::now().timestamp().to_string();
    let new_req = StoredRequest {
        // src/entities/storedrequest.rs
        method: request.method,
        host: request.host,
        port: 80,
        uri: request.uri,
        headers: request.headers,
        body: now,
    };

    let _ = write_request_to_db(new_req.clone(), pool).await; // create stored request and insert into database, ignoring the result

    Ok(())
}

#[post("/plugins/shopify", data = "<data>")]
async fn shopify_webhook<'a>(
    request: RRequest<'a>,
    pool: &rocket::State<MySqlPool>, // see line 237, wrapping this in a State<> signals Rocket to bring this into scope
    data: String,
) -> Result<(), ErrorResponder> {
    println!("{:?}", request);

    let new_req = StoredRequest {
        // src/entities/storedrequest.rs
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
async fn slow_test_server<'a>(
    // for testing purposes, https://github.com/helloimalemur/Slow-Server to simulate slow-responding server
    delay_num: i64,
    request: RRequest<'a>,
    pool: &rocket::State<MySqlPool>,
    data: String,
) -> Result<(), ErrorResponder> {
    println!("{:?}\n--- delay: {}", request, delay_num);

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

    let reque_port = settings_map
        .get("reque_service_port")
        .expect("no service port found")
        .parse::<u16>()
        .expect("cannot parse service port");

    let require_success = settings_map
        .clone()
        .get("require_success")
        .expect("could not find require_success")
        .to_string()
        .parse::<bool>()
        .unwrap();

    let remove_from_queue_on_failure = settings_map
        .clone()
        .get("remove_from_queue_on_failure")
        .expect("could not find remove_from_queue_on_failure")
        .to_string()
        .parse::<bool>()
        .unwrap();

    let config = rocket::Config {
        port: reque_port,
        address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
        ..rocket::Config::debug_default()
    };

    // setup logging request logging to file
    let stdout = ConsoleAppender::builder().build();
    #[allow(unused_variables)]
    let log_config = LogConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(log_config).unwrap();

    // logging to info
    warn!("app::requests");

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

            let mut id: i64 = 0;
            println!("{}", id);
            let mut method: String = String::new();
            println!("{}", method);
            let mut host: String = String::new();
            println!("{}", host);
            let mut uri: String = String::new();
            println!("{}", uri);
            let mut body: String = String::new();
            println!("{}", body);

            let out_ok = out.is_ok();
            if out_ok {
                let out_bind = out.expect("could not get row");

                id = out_bind.get("id");
                method = out_bind.get("method");
                uri = out_bind.get("uri");
                body = out_bind.get("body");
                host = out_bind.get("body");

                println!("{}", id);
                println!("{}", method);
                println!("{}", host);
                println!("{}", uri);
                println!("{}", body);

                let send_success = send_stored_request(
                    http_proto.clone(),
                    http_dest.clone(),
                    uri.to_string(),
                    body.to_string(),
                )
                .await;

                if send_success && require_success {
                    println!("Deleting Request: {} - {}", uri, body);
                    delete_request_from_db(id, &interval_pool).await;
                }
                if remove_from_queue_on_failure {
                    delete_request_from_db(id, &interval_pool).await;
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
        .mount("/", routes![index, shopify_webhook, slow_test_server])
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
