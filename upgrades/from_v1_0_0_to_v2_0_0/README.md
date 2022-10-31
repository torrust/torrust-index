# DB migration

With the console command `cargo run --bin upgrade` you can migrate data from `v1.0.0` to `v2.0.0`. This migration includes:

- Changing the DB schema.
- Transferring the torrent files in the dir `uploads` to the database.

## SQLite3

TODO

## MySQL8

Please,

> WARNING: MySQL migration is not implemented yet. We also provide docker infrastructure to run mysql during implementation of a migration tool.

and also:

> WARNING: We are not using a persisted volume. If you remove the volume used by the container you lose the database data.

Run the docker container and connect using the console client:

```s
./upgrades/from_v1_0_0_to_v2_0_0/docker/start_mysql.sh
./upgrades/from_v1_0_0_to_v2_0_0/docker/mysql_client.sh
```

Once you are connected to the client you can create databases with:

```s
create database torrust_v1;
create database torrust_v2;
```

After creating databases you should see something like this:

```s
mysql> show databases;
+--------------------+
| Database           |
+--------------------+
| information_schema |
| mysql              |
| performance_schema |
| sys                |
| torrust_v1         |
| torrust_v2         |
+--------------------+
6 rows in set (0.001 sec)
```

How to connect from outside the container:

```s
mysql -h127.0.0.1 -uroot -pdb-root-password
```

## Create DB for backend `v2.0.0`

You need to create an empty new database for v2.0.0.

You need to change the configuration in `config.toml` file to use MySQL:

```yml
[database]
connect_url = "mysql://root:db-root-password@127.0.0.1/torrust_v2"
```

After running the backend with `cargo run` you should see the tables created by migrations:

```s
mysql> show tables;
+-------------------------------+
| Tables_in_torrust_v2          |
+-------------------------------+
| _sqlx_migrations              |
| torrust_categories            |
| torrust_torrent_announce_urls |
| torrust_torrent_files         |
| torrust_torrent_info          |
| torrust_torrent_tracker_stats |
| torrust_torrents              |
| torrust_tracker_keys          |
| torrust_user_authentication   |
| torrust_user_bans             |
| torrust_user_invitation_uses  |
| torrust_user_invitations      |
| torrust_user_profiles         |
| torrust_user_public_keys      |
| torrust_users                 |
+-------------------------------+
15 rows in set (0.001 sec)
```

### Create DB for backend `v1.0.0`

The `upgrade` command is going to import data from version `v1.0.0` (database and `uploads` folder) into the new empty database for `v2.0.0`.

You can import data into the source database for testing with the `mysql` DB client or docker.

Using `mysql` client:

```s
mysql -h127.0.0.1 -uroot -pdb-root-password torrust_v1 < ./upgrades/from_v1_0_0_to_v2_0_0/db_schemas/db_migrations_v1_for_mysql_8.sql
```

Using dockerized `mysql` client:

```s
docker exec -i torrust-index-backend-mysql mysql torrust_v1 -uroot -pdb-root-password < ./upgrades/from_v1_0_0_to_v2_0_0/db_schemas/db_migrations_v1_for_mysql_8.sql
```

### Commands

Connect to `mysql` client:

```s
mysql -h127.0.0.1 -uroot -pdb-root-password torrust_v1
```

Connect to dockerized `mysql` client:

```s
docker exec -it torrust-index-backend-mysql mysql torrust_v1 -uroot -pdb-root-password
```

Backup DB:

```s
mysqldump -h127.0.0.1 torrust_v1 -uroot -pdb-root-password > ./upgrades/from_v1_0_0_to_v2_0_0/db_schemas/v1_schema_dump.sql
mysqldump -h127.0.0.1 torrust_v2 -uroot -pdb-root-password > ./upgrades/from_v1_0_0_to_v2_0_0/db_schemas/v2_schema_dump.sql
```
