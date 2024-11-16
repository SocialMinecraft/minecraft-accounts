use protobuf::{Message};
use anyhow::Result;
use async_nats::{Client, Subject};
use crate::proto::minecraft_account_update::ChangeMinecraftAccountResponse;

pub async fn send_change_error(nc: Client, sub: Subject, message: &str) -> Result<()> {
    let mut resp = ChangeMinecraftAccountResponse::new();
    resp.success = false;
    resp.error_message = Some(message.to_string());
    let encoded: Vec<u8> = resp.write_to_bytes()?;
    nc.publish(sub, encoded.into()).await?;
    Ok(())
}