use protobuf::{Message, MessageField};
use async_nats::Client;
use serde::Deserialize;
use crate::handlers::util::send_change_error;
use crate::proto::minecraft_account::MinecraftAccount;
use crate::proto::minecraft_account_add::AddMinecraftAccountRequest;
use crate::proto::minecraft_account_update::{ChangeMinecraftAccountResponse, MinecraftAccountChangeType, MinecraftAccountChanged};
use crate::proto::whitelist::WhitelistAccount;
use crate::store::Store;

#[tracing::instrument]
pub async fn add(db: Store, nc: Client, msg: async_nats::Message) -> anyhow::Result<()> {
    let mut request = AddMinecraftAccountRequest::parse_from_bytes(&msg.payload)?;

    if let Some(reply) = msg.reply {

        // Lookup UUID
        if request.minecraft_uuid.is_none() {
            let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", request.minecraft_username);
            let response = reqwest::get(&url).await?;

            if response.status() == reqwest::StatusCode::OK  {
                #[derive(Deserialize)]
                struct T {
                    pub id: String,
                    pub name: String,
                }
                let response = response.json::<T>().await?;
                request.minecraft_uuid = Some(response.id);
            } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                send_change_error(nc.clone(), reply, "Minecraft Account was not found").await?;
                return Ok(());
            } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                send_change_error(nc.clone(), reply, "Minecraft Account Lookup is overload, please try again in a minute").await?;
                return Ok(());
            } else {
                send_change_error(nc.clone(), reply, "Unknown error when looking up username").await?;
                return Ok(());
            }
        }


        // Check that the minecraft name is not already in use
        if db.uuid_exists(&request.minecraft_uuid.clone().unwrap()).await? {
            send_change_error(nc.clone(), reply, "Minecraft Account is already registered.").await?;
            return Ok(());
        }

        // Create account object
        let mut account = MinecraftAccount::new();
        account.minecraft_username = request.minecraft_username.clone();
        account.minecraft_uuid = request.minecraft_uuid.clone().unwrap();
        account.deprecated_first_name = request.first_name.clone();

        // Is their first account (and thus main account)
        {
            let accounts = db.get(request.user_id.clone(), request.deprecated_discord_id.clone()).await?;
            account.is_main = accounts.len() <= 0
        }

        // Try to whitelist the account - should probably be a different method of doing this.
        {
            let mut req = WhitelistAccount::new();
            req.uuid = account.minecraft_uuid.clone();
            let encoded: Vec<u8> = req.write_to_bytes()?;
            nc.request("minecraft.whitelist.add", encoded.into()).await?;
            // todo - better error handling.
            // apparently old me said this can not actually fail... as long as we get a response.
            // todo - some type of timeout?
        }

        // save account
        let account = match db.add_account(request.user_id.clone(), request.deprecated_discord_id.clone(), &account).await {
            Ok(account) => account,
            Err(e) => {
                tracing::error!("Error creating account: {:?}", e);
                send_change_error(nc.clone(), reply, "Internal Error creating account.").await?;
                return Ok(());
            }
        };

        // Build and Send Response
        let mut resp = ChangeMinecraftAccountResponse::new();
        resp.success = true;
        resp.account = MessageField::from(Some(account.clone()));
        let encoded: Vec<u8> = resp.write_to_bytes()?;
        nc.publish(reply, encoded.into()).await?;

        // Let's broadcast the account was created.
        let mut broadcast = MinecraftAccountChanged::new();
        broadcast.user_id = request.user_id;
        broadcast.deprecated_discord_id = request.deprecated_discord_id;
        broadcast.change = MinecraftAccountChangeType::ADDED.into();
        broadcast.account = MessageField::some(account);
        let encoded: Vec<u8> = broadcast.write_to_bytes()?;
        nc.publish("accounts.minecraft.changed", encoded.into()).await?;
    }

    Ok(())
}