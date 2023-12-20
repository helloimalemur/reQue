# reQue
HTTP Request queue, replay, and prioritization micro-service.
The name is a play on the word deque, request, and queue.

### For when your API can't say no
Queue requests destined for slow backend servers when responding with 429 is unacceptable.

### Intended Usage
Created to be used in a microservice based architecture, placed in-front of the slow-responding service.
Think of it as a per-service-mini-proxy that doesn't drop your requests when drinking from a firehose.

### Place reQue in front of your slow-to-consume / slow-to-respond backend servers to "trickle" incoming requests to them.
```mermaid
flowchart LR
    A[External ASYNC API Request/Webhook] -->|http/https| B(proxy)
    B --> C{reQue}
    C -->|/your/endpoint| D[This Slow Server]
    C -->|/your/other/endpoint?with=params| E[That Slow Server]
    F[database]<-->|request queue| C{reQue}
%%    C{reQue} -->|request queue| F[database]
```


### Installation assumptions/requirements
- [Rust](https://www.rust-lang.org/tools/install) is installed.
- systemd based linux distribution.

## Install
```shell
bash -e install.sh
```

## Bring your own SQL server
    docker run -p 127.0.0.1:3306:3306  --name mdb -e MARIADB_ROOT_PASSWORD=Password123! -d mariadb:latest;
    mariadb -h 127.0.0.1 -uroot -pPassword123! -e 'CREATE DATABASE reque;';
    mariadb -D reque -h 127.0.0.1 -uroot -pPassword123! -e 'CREATE TABLE `requests` (`id` int(11) NOT NULL AUTO_INCREMENT,`method` varchar(255) NOT NULL,`host` varchar(255) NOT NULL,`port` varchar(255) NOT NULL,`uri` varchar(255) NOT NULL,`headers` varchar(255) NOT NULL,`body` varchar(6255) NOT NULL,PRIMARY KEY (`id`));';
    mariadb -h 127.0.0.1 -uroot -pPassword123! -e "CREATE USER 'dev'@'%' IDENTIFIED BY 'password';";
    mariadb -h 127.0.0.1 -uroot -pPassword123! -e "GRANT ALL PRIVILEGES ON reque.* TO 'dev'@'%';";
    mariadb -h 127.0.0.1 -uroot -pPassword123! -e "FLUSH PRIVILEGES;";
    

## Create database
```sql
CREATE DATABASE reque;
```

## Create tables needed in the Database;

```sql
CREATE TABLE `requests` (`id` int(11) NOT NULL AUTO_INCREMENT,
`method` varchar(255) NOT NULL,
`host` varchar(255) NOT NULL,
`port` varchar(255) NOT NULL,
`uri` varchar(255) NOT NULL,
`headers` varchar(255) NOT NULL,
`body` varchar(6255) NOT NULL,
PRIMARY KEY (`id`));
```

## Create database user
```sql
CREATE USER 'dev'@'%' IDENTIFIED WITH sha256_password BY 'password';
CREATE USER 'dev'@'%' IDENTIFIED BY 'password';
GRANT ALL PRIVILEGES ON reque.* TO 'dev'@'%';
FLUSH PRIVILEGES;
```

## Edit config/Settings.toml
```toml
## general app, database, and external api settings
database_url = "mysql://dev:password@localhost:3306/reque"
database_name = "reque" ## name of queue database
api_key = "yourapikey" ## for api-key protection - not currently enabled on any endpoints
log_path = "log/requests.log" ## logging is currently not working
reque_service_port = "8030"
remove_from_queue_on_failure = "false"
## reque destination endpoint settings
http_proto = "http" ## protocol of final destination server
http_dest = "localhost:7780" ## final destination server
reque_interval = "3" ## slow trickling requests to dest based on interval in seconds
require_success = "false" ## receiving slow server must respond with 200
```

## test and dev;

#### 1. Start reque
```shell
cargo run
```

#### 2. Start a test server to receive the funneled requests, such as [slow server](https://github.com/helloimalemur/Slow-Server) or "./test_server.py [<port>] within this repo"

#### 3. Send requests into reque, which will be queued and funneled to the destination endpoint specified in Settings.toml
```shell
# create entry;
curl -X POST "http://127.0.0.1:8030/plugins/shopify/" -H "Content-Type: application/json" -d '{"name": "John Doe", "age": 30, "city": "New York"}'
#create a slow-server test entry;
curl -X POST "http://127.0.0.1:8030/delay/30/" -H "Content-Type: application/json" -d '{"name": "John Doe", 30, "city": "New York"}'
# create a lot of slow-server test entries;
for i in {00..500}; do curl -X POST "http://127.0.0.1:8030/delay/3/" -d "$i"; done;
```
#### 4. Observe requests being trickle funneled to the specified endpoint based on interval specified in Settings.toml

[//]: # (```sql)
[//]: # (INSERT INTO requests &#40;method, host, port, uri, headers, body&#41; VALUES &#40;"method", "host", "port", "uri", "headers", "body"&#41;;)
[//]: # (```)


## Development and Collaboration
#### Feel free to open a pull request, please run the following prior to your submission please!
    echo "Run clippy"; cargo clippy -- -D clippy::all
    echo "Format source code"; cargo fmt -- --check



### Resources
    https://www.baeldung.com/cs/tokens-vs-sessions
    https://api.rocket.rs/v0.4/rocket/http/enum.Cookies.html
    https://api.rocket.rs/v0.4/rocket/request/trait.FromRequest.html
    https://rocket.rs/v0.5-rc/guide/requests/#custom-guards
    https://api.rocket.rs/v0.5-rc/rocket/request/trait.FromRequest.html
    https://stackoverflow.com/questions/69377336/how-to-get-state-in-fromrequest-implementation-with-rocket
    https://stackoverflow.com/questions/73868771/rust-rocket-with-sqlx-test-database-endpoints
