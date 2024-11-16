use protobuf::{Message, MessageField};
use async_nats::Client;
use crate::handlers::util::send_change_error;
use crate::proto::minecraft_account_remove::RemoveMinecraftAccountRequest;
use crate::proto::minecraft_account_update::{ChangeMinecraftAccountResponse, MinecraftAccountChangeType, MinecraftAccountChanged};
use crate::proto::whitelist::UnwhitelistAccount;
use crate::store::Store;

#[tracing::instrument]
pub async fn remove(db: Store, nc: Client, msg: async_nats::Message) -> anyhow::Result<()> {
    let request = RemoveMinecraftAccountRequest::parse_from_bytes(&msg.payload)?;

    if let Some(reply) = msg.reply {

        // Verify Ownership
        let (uuid, user_id, discord_id) = {
            let uuid;
            if request.minecraft_uuid.is_some() {
                uuid = request.minecraft_uuid.unwrap();
            } else if request.deprecated_minecraft_username.is_some() {
                let account = db.minecraft_name_to_uuid(&request.deprecated_minecraft_username.unwrap()).await?;
                if account.is_some() {
                    uuid = account.unwrap();
                } else {
                    send_change_error(nc.clone(), reply, "Unknown minecraft account.").await?;
                    return Ok(());
                }
            } else {
                send_change_error(nc.clone(), reply, "Unknown minecraft account.").await?;
                return Ok(());
            }

            let (discord, user) = db.uuid_owner(&uuid).await?;
            let mut owns = false;
            if discord.is_some() {
                owns = owns || discord.clone().unwrap().eq(&request.user_id);
            }
            if user.is_some() {
                owns = owns || user.clone().unwrap().eq(&request.user_id);
            }

            if !owns {
                send_change_error(nc.clone(), reply, "Unknown minecraft account.").await?;
                return Ok(());
            }

            (uuid, user, discord)
        };

        // get the account to broadcast later
        let account = db.get_by_minecraft(&uuid).await?.unwrap();

        // Remove whitelist
        {
            let mut req = UnwhitelistAccount::new();
            req.uuid = uuid.clone();
            let encoded: Vec<u8> = req.write_to_bytes()?;
            nc.request("minecraft.whitelist.remove", encoded.into()).await?;
        }

        // Delete account
        let _deleted = match db.delete_account(&uuid).await {
            Ok(re) => re,
            Err(e) => {
                tracing::error!("Error creating account: {:?}", e);
                send_change_error(nc.clone(), reply, "Internal Error removing account.").await?;
                return Ok(());
            }
        };

        // Build and Send Response
        let mut resp = ChangeMinecraftAccountResponse::new();
        resp.success = true;
        let encoded: Vec<u8> = resp.write_to_bytes()?;
        nc.publish(reply, encoded.into()).await?;

        // Let's broadcast the account was created.
        let mut broadcast = MinecraftAccountChanged::new();
        broadcast.user_id = user_id;
        broadcast.deprecated_discord_id = discord_id;
        broadcast.change = MinecraftAccountChangeType::REMOVED.into();
        broadcast.account = MessageField::some(account);
        let encoded: Vec<u8> = resp.write_to_bytes()?;
        nc.publish("accounts.minecraft.changed", encoded.into()).await?;

        // do we need to update the users main?
        // todo - check and update this...
    }

    Ok(())
}