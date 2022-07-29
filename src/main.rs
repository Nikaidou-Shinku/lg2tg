use teloxide::prelude::*;
use atri::{commands_handler, Command};

#[tokio::main]
async fn main() {
  let bot = Bot::from_env().auto_send();

  let handler = Update::filter_message()
    .branch(
      dptree::entry()
        .filter_command::<Command>()
        .endpoint(commands_handler)
    );

  Dispatcher::builder(bot, handler)
    .enable_ctrlc_handler()
    .build()
    .dispatch().await;
}
