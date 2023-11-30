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
```


### Installation assumptions/requirements
- [Rust](https://www.rust-lang.org/tools/install) is installed.
- systemd based linux distribution.

## Install
```shell
bash -e install.sh
```

## Development and Collaboration
Please feel free to open a pull request

## Bring your own SQL server
    docker run -p 127.0.0.1:3306:3306  --name mdb -e MARIADB_ROOT_PASSWORD=Password123! -d mariadb:latest

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
database_url = "mysql://dev:password@localhost:3306/reque"
database_name = "reque"
api_key = "yourapikey"
```

## test and dev functions;
### test using a [slow server](https://github.com/helloimalemur/Slow-Server)
```shell
# create entry;
curl -X POST "http://127.0.0.1:8030/plugins/shopify/" -H "Content-Type: application/json" -d '{"name": "John Doe", "age": 30, "city": "New York"}'
#create a slow-server test entry;
curl -X POST "http://127.0.0.1:8030/delay/30/" -H "Content-Type: application/json" -d '{"name": "John Doe", 30, "city": "New York"}'
# create a lot of slow-server test entries;
for i in {00..500}; do curl -X POST "http://127.0.0.1:8030/delay/3/" -d "$i"; done;
```
```sql
INSERT INTO requests (method, host, port, uri, headers, body) VALUES ("method", "host", "port", "uri", "headers", "body");
```


### Resources
    https://www.baeldung.com/cs/tokens-vs-sessions
    https://api.rocket.rs/v0.4/rocket/http/enum.Cookies.html
    https://api.rocket.rs/v0.4/rocket/request/trait.FromRequest.html
    https://rocket.rs/v0.5-rc/guide/requests/#custom-guards
    https://api.rocket.rs/v0.5-rc/rocket/request/trait.FromRequest.html
    https://stackoverflow.com/questions/69377336/how-to-get-state-in-fromrequest-implementation-with-rocket
    https://stackoverflow.com/questions/73868771/rust-rocket-with-sqlx-test-database-endpoints
