use std::error;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "咱能接受以下指令哦~")]
pub enum Command {
  #[command(description = "显示帮助信息")]
  Help,

  #[command(description = "绑定洛谷帐号", parse_with = "split")]
  Luogu {
    username: String,
    password: String,
    token2fa: String,
  },
}

pub async fn commands_handler(
  msg: Message,
  bot: AutoSend<Bot>,
  cmd: Command,
) -> Result<(), Box<dyn error::Error + Send + Sync>> {
  let response = match cmd {
    Command::Help => {
      Command::descriptions().to_string()
    }
    Command::Luogu {
      username,
      password,
      token2fa,
    } => {
      format!("Username: {}\nPassword: {}\n2FA Token: {}", username, password, token2fa)
    }
  };

  bot.send_message(msg.chat.id, response).await?;
  Ok(())
}
