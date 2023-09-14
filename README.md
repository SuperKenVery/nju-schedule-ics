# 南哪另一课表

基于日历订阅的南京大学课表

## 使用

[这里](https://pi.tail32664.ts.net/schedule/)

## 自建服务器

使用 `docker-compose`。

1. 根据下面的示例配置文件，编写自己的配置文件。
2. 根据注释修改 `docker-compose.yml`
    * 指定好配置文件的路径
    * 指定好redis的存储路径
    * 如果需要，修改映射的端口
3. 配置好自己的反代，比如用caddy套上https


4. 然后：
    `sudo docker-compose up -d`

即可。

### 配置文件

```toml
# The URL to connect to redis.
# If you're using the provided docker-compose.yml, don't change this.
redis_url="redis://redis:6379/"

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
