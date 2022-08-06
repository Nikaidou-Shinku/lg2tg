use std::{error, io::Read};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use tokio::{fs::File, io::AsyncWriteExt};
use reqwest::Client;
use teloxide::{
  prelude::*,
  types::InputFile,
  utils::command::BotCommands,
  dispatching::dialogue::InMemStorage,
};

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "咱能接受以下指令哦~")]
pub enum Command {
  #[command(description = "开始对话")]
  Start,

  #[command(description = "显示帮助信息")]
  Help,

  #[command(description = "绑定洛谷帐号")]
  Login,
}

pub async fn commands_handler(
  bot: AutoSend<Bot>,
  msg: Message,
  cmd: Command,
  dialogue: LoginDialogue,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
  let response = match cmd {
    Command::Start => {
      "アトリは、高性能ですから!\n\n输入 /help 获取帮助！".to_string()
    }
    Command::Help => {
      Command::descriptions().to_string()
    }
    Command::Login => {
      dialogue.update(State::ReceiveLuoguUsername).await?;
      "请告诉我您的洛谷用户名。".to_string()
    }
  };

  bot.send_message(msg.chat.id, response).await?;
  Ok(())
}

#[derive(Clone, Default)]
pub enum State {
  #[default]
  Start,
  ReceiveLuoguUsername,
  ReceiveLuoguPassword {
    username: String,
  },
  ReceiveLuoguCaptcha {
    username: String,
    password: String,
    client_id: String,
    csrf_token: String,
  },
  ReceiveLuogu2FA,
}

type LoginDialogue = Dialogue<State, InMemStorage<State>>;

pub async fn receive_luogu_username(
  bot: AutoSend<Bot>,
  msg: Message,
  dialogue: LoginDialogue,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
  let response = match msg.text() {
    Some(username) => {
      dialogue.update(State::ReceiveLuoguPassword { username: username.to_string() }).await?;
      "请告诉我您洛谷账户的密码。"
    }
    None => {
      "请告诉我您的洛谷用户名。"
    }
  };

  bot.send_message(msg.chat.id, response).await?;
  Ok(())
}

async fn get_tokens(client: &Client) -> Option<(String, String)> {
  let resp = if let Ok(resp) = client.get("https://www.luogu.com.cn/auth/login").send().await {
    resp
  } else {
    return None;
  };
  let cookie_list = resp.headers().get_all("set-cookie").iter();

  for item in cookie_list {
    if item.to_str().unwrap().contains("__client_id") {
      let client_id = item.to_str().unwrap()
        .split(';').next().unwrap()
        .split('=').nth(1).unwrap()
        .to_string();

      let re = Regex::new("<meta name=\"csrf-token\" content=\"(.+?)\">").unwrap();
      let text = resp.text().await.unwrap();
      let csrf_token = re.captures(&text).unwrap().get(1).unwrap().as_str().to_string();

      return Some((client_id, csrf_token));
    }
  }

  None
}

pub async fn get_captcha() -> Result<(String, String), Box<dyn error::Error + Send + Sync>> {
  let client = Client::new();

  let tokens = if let Some(tokens) = get_tokens(&client).await {
    tokens
  } else {
    return Err("获取 client id 失败啦！".into());
  };
  let resp = client.get("https://www.luogu.com.cn/api/verify/captcha")
    .header("cookie", format!("__client_id={}", tokens.0))
    .send().await?
    .bytes().await?;
  let data: Result<Vec<_>, _> = resp.bytes().collect();
  let mut file = File::create("./captcha.jpg").await?;
  file.write_all(&data.unwrap()).await?;

  Ok(tokens)
}

pub async fn receive_luogu_password(
  bot: AutoSend<Bot>,
  msg: Message,
  dialogue: LoginDialogue,
  username: String,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
  match msg.text() {
    Some(password) => {
      match get_captcha().await {
        Ok(tokens) => {
          dialogue.update(State::ReceiveLuoguCaptcha {
            username,
            password: password.to_string(),
            client_id: tokens.0,
            csrf_token: tokens.1,
          }).await?;
          bot.send_message(msg.chat.id, "请输入下图中的验证码。").await?;
          bot.send_photo(msg.chat.id, InputFile::file("./captcha.jpg")).await?;
        }
        Err(err) => {
          dialogue.update(State::Start).await?;
          bot.send_message(msg.chat.id, format!("好像出现了什么问题…\n{}", err)).await?;
        }
      }
    }
    None => {
      bot.send_message(msg.chat.id, "请告诉我您洛谷账户的密码。").await?;
    }
  }

  Ok(())
}

#[allow(dead_code, non_snake_case)]
#[derive(Deserialize)]
struct OkResp {
  username: String,
  syncToken: String,
  locked: bool,
  redirectTo: String,
}

#[allow(dead_code, non_snake_case)]
#[derive(Deserialize)]
struct ErrResp {
  status: i32,
  data: String,
  errorMessage: String,
  trace: String,
  customData: Vec<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LoginResp {
  Ok(OkResp),
  Err(ErrResp),
}

async fn login(
  username: &str,
  password: &str,
  client_id: &str,
  csrf_token: &str,
  captcha: &str,
) -> Result<LoginResp, Box<dyn error::Error + Send + Sync>> {
  let client = Client::new();

  let resp = client.post("https://www.luogu.com.cn/api/auth/userPassLogin")
    .header("cookie", format!("__client_id={}", client_id))
    .header("referer", "https://www.luogu.com.cn/auth/login")
    .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.167 Safari/537.36")
    .header("x-csrf-token", csrf_token)
    .json(&json!({
      "username": username,
      "password": password,
      "captcha": captcha,
    }))
    .send().await?;

  Ok(resp.json().await?)
}

pub async fn receive_luogu_captcha(
  bot: AutoSend<Bot>,
  msg: Message,
  dialogue: LoginDialogue,
  (username, password, client_id, csrf_token): (String, String, String, String),
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
  match msg.text() {
    Some(captcha) => {
      match login(&username, &password, &client_id, &csrf_token, captcha).await? {
        LoginResp::Ok(resp) => {
          if resp.locked {
            dialogue.update(State::ReceiveLuogu2FA).await?;
            bot.send_message(msg.chat.id, "请告诉我您的两步验证码。").await?;
          } else {
            dialogue.update(State::Start).await?;
            bot.send_message(msg.chat.id, "登录成功！").await?;
          }
        }
        LoginResp::Err(resp) => {
          dialogue.update(State::Start).await?;
          bot.send_message(msg.chat.id, format!("登录失败了…\n{}：{}", resp.status, resp.errorMessage)).await?;
        }
      }
    }
    None => {
      bot.send_message(msg.chat.id, "请输入上图中的验证码。").await?;
    }
  }

  Ok(())
}

// TODO: finish this
pub async fn receive_luogu_2fa(
  bot: AutoSend<Bot>,
  msg: Message,
  dialogue: LoginDialogue,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
  Ok(())
}
