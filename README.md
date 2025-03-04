# 南哪另一课表

把南京大学课程表转换为可以订阅的日历链接，方便自动更新、多端同步。

此外，系统日历app通常具有很好的整合，比如桌面小组件、锁屏小组件、语音助手整合、手表同步等功能。

## 亮点

- **一次登陆，自动更新。** 后面再退补选都不需要重新登陆/重新导入，等你的日历app自动更新或者手动刷新一下即可看到新的课表。到了下一学期时，也不需要重新登陆/导入。
- **一个链接，处处使用。** 你可以在自己的任何设备上使用同一个订阅链接，最新的课表便会同步到各个设备上。你也可以把链接**分享给朋友**，告诉ta上课的时候别来烦我！
- **跨平台兼容。** iCalendar格式是个通用的标准，南哪课表不再局限于手机。电脑、平板、手表，甚至电视、机顶盒都可以看课程表啦～iOS/Android可用，Windows/macOS/Linux也可用。HarmonyOS NEXT暂时不支持.ics文件的打开，第三方应用暂时也未实现支持。（对于鸿蒙NEXT用户，可以将链接改成http访问下载ics文件以后导入鸿蒙设备，然后云同步至鸿蒙NEXT设备）
- **更好的系统整合。** 此处的日历订阅通常导入到系统自带的日历，而系统自带的日历对系统里各种功能的支持肯定是特别完善的。比如桌面小组件、锁屏小组件、语音助手等功能都能安排上。
- **地图导航。** 看到了教室名字却不知道在哪里？没关系，日程中带上了地图！只要点一下即可跳转到导航app，手把手带你前往教室！（目前只支持苹果系统，只支持仙林校区）

<img width="300px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/53ee7918-d1aa-4ba8-aa61-0f27e6e85f92">
<img width="300px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/26d8cd25-ae52-4998-9f51-7878ea74ae17">

<img width="500px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/f551c18c-f113-40cb-b345-3a23cebbc4e8">
<img width="400px" alt="图片" src="https://github.com/SuperKenVery/nju-schedule-ics/assets/39673849/b03b2857-47e7-48c5-8e26-e55b56573ac1">

登陆界面：

<img width="915" alt="Login" src="https://github.com/user-attachments/assets/f5bd31f0-9169-4d90-ae64-8d89de65bd62">


## 使用提供的服务器

[这里](https://n100.tail32664.ts.net/schedule/)

[旧的（连不上了）](https://pi.tail32664.ts.net/schedule/)

## 隐私与数据安全

此项目用于搭建一个服务器（以下称为“本服务器"），该服务器会将南大的课程表转换为iCalendar日历格式。为了从南大的服务器获取课程表，本服务器需要拿到你的统一认证用户名和密码以登陆。

我们不会存储你提交的用户名、密码，但会存储登陆过程中产生的cookie以确保后续更新。该cookie对你帐号的操作权限约等于密码，拿到了cookie就等同于登陆后的状态。

日历的订阅链接不会包含除了课表外的任何其他隐私信息。

我们无意偷盗你的帐号，也尽力编写保护隐私的代码。但就像任何程序一样，我们无法保证没有bug。此外，作为一个开源软件，它不提供任何担保，使用过程中的任何风险由用户自行承担。

## 自建服务器

### 安装服务端

1. 可以直接运行nix flake：

```bash
cachix use superkenvery  # 使用我的构建缓存，避免编译
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

## 参与开发

### 关于地图
在iCalendar标准中，与事件位置相关的字段有两个：[LOCATION](https://www.kanzaki.com/docs/ical/location.html)和[GEO](https://www.kanzaki.com/docs/ical/geo.html)。
- `LOCATION`是个文本，是地点的名字。给人看很方便，但电脑要想知道它具体在哪就只能在地图里搜。但是南大很多内部地点地图上根本搜不到，就会跑到奇奇怪怪的地方去。
- `GEO`字段是一个浮点数对，表示地点的经纬度。这个用来指定位置就很精确了，**可惜我所见过的日历app似乎都没有读取它**。

在标准之外，苹果使用了一个扩展字段：`X-APPLE-STRUCTURED-LOCATION`。
- 这个字段的`X-ADDRESS`属性指定了地点名字的第二行
- `X-TITLE`指定了日历中地点显示的第一行
- 这个字段的值提供了地点的经纬度

当日历的事件提供了这个字段及对应参数时，在苹果系统的日历app中就会显示一张地图，点一下就可以跳转到导航。

但很可惜的是，苹果的日历是我见过的唯一一个导出日历再导入后还能保留地图的。如果导出了再导入就没有地图了，那大概就表示那个地图不支持从ics文件中加载地图，那再研究也没什么意义了。如果大家发现自己的日历客户端有根据ics文件内容显示地图的功能，欢迎告诉我，让我们在更多的日历中显示出地图！

此外，目前发现macOS和iOS上同样的经纬度可能会漂移到不同的地点，具体情况和原因我还不是很清楚。

### 缺失地点
日历文件中不仅会制定时间发生地点的名称，还可以指定经纬度。如果只有名称，日历app就只能在地图上搜索这个名字，但是南大内部的很多地方在地图上就搜不出来，所以就会出现很奇怪的地点。但如果指定了经纬度，就可以非常精确地指定地点了。

在本项目中，名字是从课表信息中直接提取出来的，经纬度则是我做了一个映射，在`src/schedule/location.rs`中。不要害怕，这个文件超级好读：
```rust
} else if location.contains("化学楼") {
    GeoLocation::new(32.118459, 118.952461).into()
} else if location.contains("环科楼") {
    GeoLocation::new(32.117099, 118.953059).into()
}
```
大概就是长这样，你要是想加一个地点就照葫芦画瓢就行了，直接复制粘贴，那两个数字就是经纬度，改了就行了。

那么怎样获取一个地点的经纬度呢？我的做法是这样的（但显然有其他的方法）：
- 在iOS中打开地图app，把要的地方放在屏幕地图区域的中心
- 点击地图app的搜索框，不要输入任何内容，点一下框然后点拷贝
- 此时会得到一个像这样的链接： `https://maps.apple.com/?ll=32.118459,118.952461&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001545,0.001942&t=m`。显然，`ll=32.118459,118.952461`就是经纬度了。

### 适配其他学校
本项目设计时秉承模块化思想，参见下面的“项目架构”，如果要适配新学校应该是不太难的。

大概来说，本项目分为以下三个部份：

1. 与学校服务器对接，把课程信息转换为`Vec<Course>`。在本项目中是`src/nju`，与南大服务器对接。如果要适配新学校，这里应该是改动最大的地方。
2. 把课程信息转换为iCalendar格式。在本项目中是`src/schedule`，适配新学校时应该基本不用改。
3. 服务端，在访问时返回网页并把对应人的ics文件响应回去。在本项目中是`src/server`，适配新学校时应该基本不用改。

如果要适配新学校，就改第一部份应该就可以了。如果有问题欢迎提issue/discussion或者加QQ群讨论。

### 项目架构

html：静态网页资源

nju：对接南京大学服务器

schedule：把南大服务器返回的数据转换为ics文件

server：服务端

更多内容可见 `mod.rs`的注释，也欢迎GitHub Discussion/QQ群直接问～
