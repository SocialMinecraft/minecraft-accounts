use protobuf::Message;
use async_nats::Client;
use crate::proto::minecraft_account_list::{ListMinecraftAccountsRequest, ListMinecraftAccountsResponse};
use crate::store::Store;

#[tracing::instrument]
pub async fn list(db: Store, nc: Client, msg: async_nats::Message) -> anyhow::Result<()> {
    let request = ListMinecraftAccountsRequest::parse_from_bytes(&msg.payload)?;

    if let Some(reply) = msg.reply {

        let accounts = db.get(Some(request.user_id.clone()), Some(request.user_id.clone())).await?;

        // Build and Send Response
        let mut resp = ListMinecraftAccountsResponse::new();
        resp.user_id = request.user_id.clone();
        resp.accounts = accounts;
        let encoded: Vec<u8> = resp.write_to_bytes()?;
        nc.publish(reply, encoded.into()).await?;

    }

    Ok(())
}