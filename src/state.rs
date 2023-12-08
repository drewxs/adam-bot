use reqwest::Client as HttpClient;
use serenity::gateway::ShardManager;
use songbird::typemap::TypeMapKey;
use std::sync::Arc;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}
