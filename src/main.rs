mod proto;
mod util;

use anyhow::Result;
use async_nats::Client;
use protobuf::Message;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<()> {

    // get the app name, used for group and such
    let app_name = match util::get_app_name() {
        Some(name) => name,
        None => { return Err(anyhow::anyhow!("Could not  determine application name.")); },
    };

    // Setup logging
    util::setup_logging(app_name.as_str());

    // connect to db
    let _db = util::connect_to_database().await?;

    // connect to nats
    let nc = util::connect_to_nats().await?;

    //let mut set = JoinSet::new();

    let _nc = nc.clone();
    /* Or * /
    set.spawn(async move {
        util::handle_requests(_nc, "vault.store", move|_nc, msg| {
            handle_hello(/ *[other params]* /, _nc, msg)
        }).await.expect("hello");
    });*/

    //set.join_all().await;
    Ok(())
}
