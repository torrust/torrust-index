# Upgrade from v1.0.0 to v2.0.0

## How-to

To upgrade from version `v1.0.0` to `v2.0.0` you have to follow these steps:

- Back up your current database and the `uploads` folder. You can find which database and upload folder are you using in the `Config.toml` file in the root folder of your installation.
- Set up a local environment exactly as you have it in production with your production data (DB and torrents folder).
- Run the application locally with: `cargo run`.
- Execute the upgrader command: `cargo run --bin upgrade ./data.db ./data_v2.db ./uploads`
- A new SQLite file should have been created in the root folder: `data_v2.db`
- Stop the running application and change the DB configuration to use the newly generated configuration:

```toml
[database]
connect_url = "sqlite://data_v2.db?mode=rwc"
```

- Run the application again.
- Perform some tests.
- If all tests pass, stop the production service, replace the DB, and start it again.

## Tests

Before replacing the DB in production you can make some tests like:

- Try to log in with a preexisting user. If you do not know any you can create a new "test" user in production before starting with the upgrade process. Users had a different hash algorithm for the password in v1.
- Try to create a new user.
- Try to upload and download a new torrent containing a single file (with and without md5sum).
- Try to upload and download a new torrent containing a folder.

## Notes

The `db_schemas` contains the snapshots of the source and target databases for this upgrade.
