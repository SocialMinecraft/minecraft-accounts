use anyhow::Result;
use protobuf::SpecialFields;
use sqlx::PgPool;
use sqlx::postgres::PgQueryResult;
use crate::proto::minecraft_account::MinecraftAccount;

#[derive(Clone, Debug)]
pub struct Store {
    db: PgPool
}

struct T {
    pub id: i64,

    discord_id: Option<String>,
    user_id: Option<String>,

    minecraft_uuid: String,
    minecraft_username: String,

    is_main: bool,

    first_name: Option<String>, // deprecated
}

impl Store {

    pub fn new(db: PgPool) -> Self {
        Store { db }
    }

    pub async fn add_account(&self, user_id: Option<String>, discord_id: Option<String>, account: &MinecraftAccount) -> Result<MinecraftAccount> {

        let re : sqlx::Result<T> = sqlx::query_as!(
            T,
            r#"
            INSERT INTO accounts (
                discord_id, user_id,
                minecraft_uuid, minecraft_username,
                is_main,
                first_name
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id,
                discord_id, user_id,
                minecraft_uuid, minecraft_username,
                is_main,
                first_name
            ;"#,
            discord_id,
            user_id,
            account.minecraft_uuid,
            account.minecraft_username,
            account.is_main,
            account.deprecated_first_name,
        )
            .fetch_one(&self.db)
            .await;

        let re = re?;
        Ok(MinecraftAccount{
            deprecated_first_name: re.first_name.unwrap_or("Deprecated".to_string()),

            minecraft_uuid: re.minecraft_uuid,
            minecraft_username: re.minecraft_username,
            is_main: re.is_main,

            special_fields: SpecialFields::default(),
        })
    }

    pub async fn update_account(&self, account: &MinecraftAccount) -> Result<MinecraftAccount> {

        let re : sqlx::Result<T> = sqlx::query_as!(
            T,
            r#"
            UPDATE
                accounts
            SET
                minecraft_username = $2,
                is_main = $3
            WHERE
                minecraft_uuid = $1
            RETURNING
                id,
                discord_id, user_id,
                minecraft_uuid, minecraft_username,
                is_main,
                first_name
            ;"#,
            account.minecraft_uuid,
            account.minecraft_username,
            account.is_main,
        )
            .fetch_one(&self.db)
            .await;

        let re = re?;
        Ok(MinecraftAccount{
            deprecated_first_name: re.first_name.unwrap_or("Deprecated".to_string()),

            minecraft_uuid: re.minecraft_uuid,
            minecraft_username: re.minecraft_username,
            is_main: re.is_main,

            special_fields: SpecialFields::default(),
        })
    }

    pub async fn delete_account(&self, minecraft_uuid: &String) -> Result<bool> {

        let re : sqlx::Result<PgQueryResult> = sqlx::query!(
            r#"
            DELETE FROM accounts
            WHERE minecraft_uuid = $1
            ;"#,
            minecraft_uuid
        )
            .execute(&self.db)
            .await;

        let re = re?;

        Ok(re.rows_affected() == 1)
    }

    pub async fn uuid_exists(&self, minecraft_uuid: &String) -> Result<bool> {
        struct T2 {
            minecraft_uuid: String,
        }
        let re : sqlx::Result<Option<T2>> = sqlx::query_as!(
            T2,
            r#"
            SELECT minecraft_uuid FROM accounts WHERE minecraft_uuid = $1
            ;"#,
            minecraft_uuid
        )
            .fetch_optional(&self.db)
            .await;

        let re = re?;

        Ok(re.is_some())
    }

    pub async fn get_by_user(&self, id: &String) -> Result<Vec<MinecraftAccount>> {

        let re : sqlx::Result<Vec<T>> = sqlx::query_as!(
            T,
            r#"
            SELECT
                id,
                discord_id, user_id,
                minecraft_uuid, minecraft_username,
                is_main,
                first_name
            FROM
                accounts
            WHERE user_id = $1
            ;"#,
            id
        )
            .fetch_all(&self.db)
            .await;

        let re = re?;

        let re = re.into_iter().map(|t| MinecraftAccount{
            deprecated_first_name: t.first_name.unwrap_or("Deprecated".to_string()),

            minecraft_uuid: t.minecraft_uuid,
            minecraft_username: t.minecraft_username,
            is_main: t.is_main,

            special_fields: SpecialFields::default(),
        }).collect();

        Ok(re)
    }

    pub async fn get_by_discord(&self, id: &String) -> Result<Vec<MinecraftAccount>> {

        let re : sqlx::Result<Vec<T>> = sqlx::query_as!(
            T,
            r#"
            SELECT
                id,
                discord_id, user_id,
                minecraft_uuid, minecraft_username,
                is_main,
                first_name
            FROM
                accounts
            WHERE discord_id = $1
            ;"#,
            id
        )
            .fetch_all(&self.db)
            .await;

        let re = re?;

        let re = re.into_iter().map(|t| MinecraftAccount{
            deprecated_first_name: t.first_name.unwrap_or("Deprecated".to_string()),

            minecraft_uuid: t.minecraft_uuid,
            minecraft_username: t.minecraft_username,
            is_main: t.is_main,

            special_fields: SpecialFields::default(),
        }).collect();

        Ok(re)
    }

    pub async fn get(&self, user: Option<String>, discord: Option<String>) -> Result<Vec<MinecraftAccount>> {
        let mut re = Vec::new();
        for account in self.get_by_user(&user.unwrap()).await? {
            re.push(account);
        }
        for account in self.get_by_discord(&discord.unwrap()).await? {
            re.push(account);
        }
        Ok(re)
    }


}