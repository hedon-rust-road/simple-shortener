# Simple Shortener

### Start a MySQL server by Docker
```bash
docker run --name mysql -v /Users/wangjiahan/mysql:/var/lib/mysql -p 3306:3306 -e MYSQL_ROOT_PASSWORD=root -d mysql:latest
```
Connect to it:
```bash
mysql -h 127.0.0.1 -P 3306 -u root -p
```

### Create `shortener` Database
```bash
CREATE DATABASE shortener;
```