use crate::entities::storedrequest::StoredRequest;
use sqlx::{MySql, MySqlPool, Pool};

// write created request to db
pub async fn write_request_to_db(request: StoredRequest<'_>, pool: &rocket::State<MySqlPool>) {
    let req = request.clone();
    let _insert = sqlx::query(
        "INSERT INTO requests (method, host, port, uri, headers, body)
        VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(req.method)
    .bind(req.host)
    .bind(req.port)
    .bind(req.uri)
    .bind("")
    .bind(req.body)
    .execute(&**pool)
    .await
    .unwrap();
}

pub async fn delete_request_from_db(id: i64, pool: &Pool<MySql>) {
    let _delete = sqlx::query("DELETE FROM requests WHERE (id)=?")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn send_stored_request(
    http_proto: String,
    http_dest: String,
    uri: String,
    body: String,
) -> bool {
    let built_uri = format!("{}://{}{}", http_proto, http_dest, uri);
    println!("Sending Request;\n{}", built_uri);
    println!("{}\n", body);
    let res = reqwest::Client::new()
        .post(built_uri)
        .body(body)
        .send()
        .await;

    let mut success = false;
    if res.is_ok() {
        match res.unwrap().status().as_u16() {
            200 => success = true,
            _ => success = false,
        }
    }
    success
}
