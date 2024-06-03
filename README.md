# Simple Shortener

### Preparation
#### Start a MySQL server by Docker
```bash
docker run --name mysql -v /Users/wangjiahan/mysql:/var/lib/mysql -p 3306:3306 -e MYSQL_ROOT_PASSWORD=root -d mysql:latest
```
Connect to it:
```bash
mysql -h 127.0.0.1 -P 3306 -u root -p
```
#### Create `shortener` Database
```bash
CREATE DATABASE shortener;
```

### Usage
#### shorten
```http
POST http://localhost:9090/
Content-Type: application/json

{
  "url": "https://time.geekbang.org"
}
```

#### redirect
```http
GET http://localhost:9090/{id}
```
Note: replace the `{id}` with the id you get from the response of `shorten`.