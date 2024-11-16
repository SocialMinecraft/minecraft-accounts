mod proto;
mod util;
mod store;
mod handlers;

use anyhow::Result;
use tokio::task::JoinSet;
use crate::handlers::add::add;
use crate::handlers::list::list;
use crate::handlers::remove::remove;
use crate::store::Store;

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
    let db = util::connect_to_database().await?;
    let store = Store::new(db.clone());

    // connect to nats
    let nc = util::connect_to_nats().await?;

    let mut set = JoinSet::new();

    let _nc = nc.clone();
    let _store = store.clone();
    set.spawn(async move {
        util::handle_requests(_nc, "accounts.minecraft.add", move|_nc, msg| {
            add(_store.clone(), _nc, msg)
        }).await.expect("accounts.minecraft.add");
    });

    let _nc = nc.clone();
    let _store = store.clone();
    set.spawn(async move {
        util::handle_requests(_nc, "accounts.minecraft.remove", move|_nc, msg| {
            remove(_store.clone(), _nc, msg)
        }).await.expect("accounts.minecraft.remove");
    });

    let _nc = nc.clone();
    let _store = store.clone();
    set.spawn(async move {
        util::handle_requests(_nc, "accounts.minecraft.list", move|_nc, msg| {
            list(_store.clone(), _nc, msg)
        }).await.expect("accounts.minecraft.list");
    });

    set.join_all().await;
    Ok(())
}
