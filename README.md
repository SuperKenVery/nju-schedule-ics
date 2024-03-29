# 南哪另一课表

基于日历订阅的南京大学课表


<img width="300px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/53ee7918-d1aa-4ba8-aa61-0f27e6e85f92">
<img width="300px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/26d8cd25-ae52-4998-9f51-7878ea74ae17">

<img width="500px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/f551c18c-f113-40cb-b345-3a23cebbc4e8">
<img width="400px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/b03b2857-47e7-48c5-8e26-e55b56573ac1">

登陆界面：

<img width="915" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/9d3a7dad-d1a1-4bbe-8e05-8246329e8289">


## 使用

[这里](https://pi.tail32664.ts.net/schedule/)

## 自建服务器

```bash
nix run github:SuperKenVery/nju-schedule-ics -- --config config.toml
```

如果指定的文件不存在则会生成默认的配置文件并退出。

也可以使用docker:

```bash
nix build .#docker
docker load -i result
touch config.toml # Without this docker would create a directory
docker run -p 8899:8899 -v ./config.toml:/config.toml nju-schedule-ics:<see tag with docker images>
```

也可以从源码编译：

```bash
cargo run -- --config config.toml
```

nix cache:

```bash
cachix use superkenvery
```

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
