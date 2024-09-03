# 南哪另一课表

把南京大学课程表转换为可以订阅的日历链接，方便自动更新、多端同步。

此外，系统日历app通常具有很好的整合，比如桌面小组件、锁屏小组件、语音助手整合、手表同步等功能。

## 亮点

- **一次登陆，自动更新。** 后面再退补选都不需要重新登陆/重新导入，等你的日历app自动更新或者手动刷新一下即可看到新的课表。到了下一学期时，也不需要重新登陆/导入。
- **一个链接，处处使用。** 你可以在自己的任何设备上使用同一个订阅链接，最新的课表便会同步到各个设备上。你也可以把链接**分享给朋友**，告诉ta上课的时候别来烦我！
- **跨平台兼容。** iCalendar格式是个通用的标准，南哪课表不再局限于手机。电脑、平板、手表，甚至电视、机顶盒都可以看课程表啦～iOS/Android可用，Windows/macOS/Linux也可用，且即将到来的鸿蒙next很可能也可以用（虽然我没试过）
- **更好的系统整合。** 此处的日历订阅通常导入到系统自带的日历，而系统自带的日历对系统里各种功能的支持肯定是特别完善的。比如桌面小组件、锁屏小组件、语音助手等功能都能安排上。
- **地图导航。** 看到了教室名字却不知道在哪里？没关系，日程中带上了地图！只要点一下即可跳转到导航app，手把手带你前往教室！（目前只支持苹果系统，只支持仙林校区）

<img width="300px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/53ee7918-d1aa-4ba8-aa61-0f27e6e85f92">
<img width="300px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/26d8cd25-ae52-4998-9f51-7878ea74ae17">

<img width="500px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/f551c18c-f113-40cb-b345-3a23cebbc4e8">
<img width="400px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/b03b2857-47e7-48c5-8e26-e55b56573ac1">

登陆界面：

<img width="915" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/9d3a7dad-d1a1-4bbe-8e05-8246329e8289">

## 使用提供的服务器

[这里](https://pi.tail32664.ts.net/schedule/)

## 隐私与数据安全

此项目用于搭建一个服务器（以下称为“本服务器"），该服务器会将南大的课程表转换为iCalendar日历格式。为了从南大的服务器获取课程表，本服务器需要拿到你的统一认证用户名和密码以登陆。

我们不会存储你提交的用户名、密码，但会存储登陆过程中产生的cookie以确保后续更新。该cookie对你帐号的操作权限约等于密码，拿到了cookie就等同于登陆后的状态。

日历的订阅链接不会包含除了课表外的任何其他隐私信息。

我们无意偷盗你的帐号，也尽力编写保护隐私的代码。但就像任何程序一样，我们无法保证没有bug。此外，作为一个开源软件，它不提供任何担保，使用过程中的任何风险由用户自行承担。

## 自建服务器

1. 可以直接运行nix flake：

```bash
nix run github:SuperKenVery/nju-schedule-ics -- --config config.toml
```

如果指定的文件不存在则会生成默认的配置文件并退出。

2. 也可以使用docker部署:

```bash
nix build github:SuperKenVery/nju-schedule-ics#docker
docker load -i ./result # You will get a tag here
touch config.toml # Without this docker would create a directory
docker run -p 8899:8899 -v ./config.toml:/config.toml nju-schedule-ics:<use the previous tag>
```

3. 也可以从源码编译并运行：

```bash
cargo run -- --config config.toml
```



如果使用nix构建，可以添加nix cache来避免编译，直接下载二进制:

```bash
cachix use superkenvery
```

目前GitHub Actions会生成`x86_64-linux`和`x86_64-darwin`的cache，我有时会推送 `aarch64-darwin`和 `aarch64-linux`的cache。

### 配置文件

```toml
# The path to SQLite database
# which stores cookies
db_path="./cookies.sqlite"

# The URL this site is hosted
# No trailing slash
# Must start with https://
site_url="https://example.com/sub_dir"

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
