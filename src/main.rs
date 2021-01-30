use error_chain::error_chain;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use regex::Regex;
use bytes::Bytes;

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
         ZipArchive(zip::result::ZipError);
     }
}

/// 获取配置
async fn get_config() -> Result<String> {
    let mut json_file = File::open("./iconfont.json")?;
    let mut json_str = String::new();
    println!("开始读取");
    json_file.read_to_string(&mut json_str)?;
    println!("读取完成");
    return Ok(json_str)
}

/// 下载压缩包
async fn download(pid: String, cookie_value: String) -> Result<Bytes> {
    // 客户端
    let client = reqwest::Client::new();
    let target = String::from("https://www.iconfont.cn/api/project/download.zip?pid=") + &pid;
    let cookie = String::from("EGG_SESS_ICONFONT=") + &cookie_value;
    println!("开始下载");
    let response = client.get(&target).header("cookie", &cookie).send().await?;
    println!("下载完成");

    // 内容
    let content =  response.bytes().await?;

    Ok(content)
}

/// 解压压缩包
async fn unzip(content: Bytes, target_path: String) -> Result<()> {
    let reader = std::io::Cursor::new(&content);
    let mut zip = zip::ZipArchive::new(reader)?;
    let re = Regex::new(r"iconfont\.js$").unwrap();
    for i in 0..zip.len()
    {
        let mut file = zip.by_index(i).unwrap();
        println!("Is dir: {}, Filename: {}", file.is_dir(), file.name());
        if re.is_match(&file.name()) {
            println!("需要导出: {}", file.name());
            let mut str = String::from("");
            let usize = file.read_to_string(&mut str).unwrap();
            println!("文件大小: {}", &usize);
            let file_uri = String::from(target_path) + "iconfont.js";
            let iconfont_file_path = Path::new(&file_uri);
            let mut iconfont_file = match File::create(&iconfont_file_path) {
                Err(why) => panic!("无法创建写入文件 {}", why),
                Ok(file) => file,
            };
            iconfont_file.write_all(&str.as_bytes())?;
            break;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("读取准备");
    let json_str = get_config().await?;

    println!("准备解析配置文件");
    let config = json::parse(&json_str).unwrap();
    println!("配置文件解析完成");

    let pid = &config["pid"];
    let cookie = &config["cookie"];
    let path = &config["path"];
    println!("解析到的pid为: {}, 准备开始下载", &pid);

    let content = download(pid.to_string(), cookie.to_string()).await?;
    unzip(content, path.to_string()).await?;
    Ok(())
}

