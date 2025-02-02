use protobuf::{Message, MessageField};
use async_nats::Client;
use crate::proto::minecraft_account_get::{GetMinecraftAccountRequest, GetMinecraftAccountResponse};
use crate::store::Store;

#[tracing::instrument]
pub async fn get(db: Store, nc: Client, msg: async_nats::Message) -> anyhow::Result<()> {
    let request = GetMinecraftAccountRequest::parse_from_bytes(&msg.payload)?;

    if let Some(reply) = msg.reply {


        let account = db.get_by_minecraft(&request.minecraft_uuid).await?;

        // Build and Send Response
        let mut resp = GetMinecraftAccountResponse::new();

        if account.is_some() {
            resp.account_found = true;

            let account = account.unwrap();
            // resp.user_id = request.user_id.clone();
            resp.account = MessageField::some(account);
        } else {
            resp.account_found = false;
        }

        let encoded: Vec<u8> = resp.write_to_bytes()?;
        nc.publish(reply, encoded.into()).await?;
    }

    Ok(())
}