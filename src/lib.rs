use std::{error, io::Read};
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

async fn get_client_id(client: &Client) -> Option<String> {
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
        .split('=').nth(1).unwrap();
      return Some(client_id.to_string());
    }
  }

  None
}

pub async fn get_captcha() -> Result<String, Box<dyn error::Error + Send + Sync>> {
  let client = Client::new();

  let client_id = if let Some(id) = get_client_id(&client).await {
    id
  } else {
    return Err("获取 client id 失败啦！".into());
  };
  let resp = client.get("https://www.luogu.com.cn/api/verify/captcha")
    .header("cookie", format!("__client_id={}", client_id))
    .send().await?
    .bytes().await?;
  let data: Result<Vec<_>, _> = resp.bytes().collect();
  let mut file = File::create("./captcha.jpg").await?;
  file.write_all(&data.unwrap()).await?;

  Ok(client_id)
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
        Ok(client_id) => {
          dialogue.update(State::ReceiveLuoguCaptcha {
            username,
            password: password.to_string(),
            client_id: client_id.to_string(),
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
  };

  Ok(())
}
