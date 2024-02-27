# 南哪另一课表

基于日历订阅的南京大学课表

## 使用

[这里](https://pi.tail32664.ts.net/schedule/)

## 自建服务器

```bash
cargo run -- --config config.toml
```

如果指定的文件不存在则会生成默认的配置文件并退出。

也可以使用提供的Dockerfile和docker compose。

### 配置文件

```toml
# The path to SQLite database
# which stores cookies
db_path="./cookies.sqlite"

# The base URL of this site
# Don't add the trailing slash
site_url="https://example.com/example/sub/directory"

# Listen address&port
# This is different from site_url, as you'll probably
# use a reverse proxy in front of this.
listen_addr="0.0.0.0:8899"
```

## 项目架构

html：静态网页资源

nju：对接南京大学服务器

schedule：把南大服务器返回的数据转换为ics文件

server：服务端

更多内容可见 `mod.rs`的注释
