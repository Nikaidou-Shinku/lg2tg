use std::sync::Arc;
use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue::ErasedStorage;

#[derive(Deserialize, Serialize)]
pub struct AtriState {

}

pub type AtriStorage = Arc<ErasedStorage<AtriState>>;
