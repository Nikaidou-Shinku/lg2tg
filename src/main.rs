mod storage;

use teloxide::{
  prelude::*,
  dispatching::dialogue::{
    serializer::Json,
    SqliteStorage,
    Storage,
    InMemStorage,
  },
};
use storage::AtriStorage;
use atri::{
  Command,
  commands_handler,
  State,
  receive_luogu_username,
  receive_luogu_password,
};

#[tokio::main]
async fn main() {
  let bot = Bot::from_env().auto_send();

  let _storage: AtriStorage = SqliteStorage::open("data.db", Json).await.unwrap().erase();

  let handler = Update::filter_message()
    .enter_dialogue::<Message, InMemStorage<State>, State>()
    .branch(
      dptree::case![State::Start]
        .filter_command::<Command>()
        .endpoint(commands_handler)
    )
    .branch(
      dptree::case![State::ReceiveLuoguUsername]
        .endpoint(receive_luogu_username)
    )
    .branch(
      dptree::case![State::ReceiveLuoguPassword { username }]
        .endpoint(receive_luogu_password)
    );

  Dispatcher::builder(bot, handler)
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch().await;
}
