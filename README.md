# Minecraft Account Repo

## Updating from the prev service
```sql
CREATE TABLE "public"."_sqlx_migrations" (
    "version" bigint NOT NULL,
    "description" text NOT NULL,
    "installed_on" timestamptz DEFAULT now() NOT NULL,
    "success" boolean NOT NULL,
    "checksum" bytea NOT NULL,
    "execution_time" bigint NOT NULL,
    CONSTRAINT "_sqlx_migrations_pkey" PRIMARY KEY ("version")
) WITH (oids = false);

INSERT INTO "_sqlx_migrations" ("version", "description", "installed_on", "success", "checksum", "execution_time") VALUES
(20241116173545,	'create',	'2024-11-16 18:03:16.748234+00',	't',	'\x1b0309409500658da98fc5a61c83e0bbcfccdc71fff66226cffa8f95f78321ea261b066d6540c4dd08aa2caa4c44c1d5',	8288708);
```

## Creating a release

```sh
cargo release patch/minor/major --execute
````

##  Creating a sql migration

```sh
sqlx migrate add
```

## Update sql scripts for release

This needs to be ran and commited to the repo to allow
for the ci/cd to build with sql.

```shell
cargo sqlx prepare
```