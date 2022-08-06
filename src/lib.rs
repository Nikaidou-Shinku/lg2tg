use std::error;
use teloxide::{
  prelude::*,
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
  ReceiveLuogu2FA {
    username: String,
    password: String,
  },
}

type LoginDialogue = Dialogue<State, InMemStorage<State>>;

pub async fn receive_luogu_username(
  bot: AutoSend<Bot>,
  msg: Message,
  dialogue: LoginDialogue,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
  match msg.text() {
    Some(username) => {
      bot.send_message(msg.chat.id, "请告诉我您洛谷账户的密码。").await?;
      dialogue.update(State::ReceiveLuoguPassword { username: username.to_string() }).await?;
    }
    None => {
      bot.send_message(msg.chat.id, "请告诉我您的洛谷用户名。").await?;
    }
  }
  Ok(())
}

pub async fn receive_luogu_password(
  bot: AutoSend<Bot>,
  msg: Message,
  dialogue: LoginDialogue,
  username: String,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
  match msg.text() {
    Some(password) => {
      bot.send_message(msg.chat.id, format!("将绑定以下洛谷账户\n用户名：{}\n密码：{}", username, password)).await?;
      // TODO: login and check 2FA
      dialogue.update(State::Start).await?;
    }
    None => {
      bot.send_message(msg.chat.id, "请告诉我您洛谷账户的密码。").await?;
    }
  }
  Ok(())
}
