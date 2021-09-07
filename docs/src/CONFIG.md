# Configuring Torrust
Torrust's configuration is a simple TOML file. If no TOML file is found, it will fail on startup.

## Must change
> These are all of the configuration oprtions that can affect the security of your instance. Please make sure to change these to your own values.
- `tracker.token`
- `auth.secret_key`

## Configuration

### `REQUIRED` `[tracker]` Section
- `REQUIRED` `url`: public UDP url of the Torrust Tracker instance.
- `REQUIRED` `api_url`: URL of the Torrust Tracker API, usually `http://localhost:1212`.
- `REQUIRED` `token`: token configured in the Torrust Tracker configuration.
- `REQUIRED` `token_valid_seconds`: Lifetime of a tracker key.

### `REQUIRED` `[net]` Section
- `REQUIRED` `port`: The port the API will listen on. It's not advised to use ports under 1024 because root access is required for these ports.

### `REQUIRED` `[database]` Section
- `REQUIRED` `connect_url`: The connection URL of the database. Should always start with `sqlite:`, no other databases are supported as of now.Including `mode=rwc` allows the database to be `Read / Writed / Created`. Example: `sqlite://data.db?mode=rwc`
- `REQUIRED` `torrent_info_update_interval`: Interval in seconds for updating torrent seeder and leecher information. This can be a heavy operation depending on the amount of torrents that are tracked, and thus is not recommended to be lower than `1800` seconds.

### `REQUIRED` `[mail]` Section
- `REQUIRED` `server`: Hostname or IP address of a SMTP server.
- `REQUIRED` `port`: Port of the SMTP server.
- `REQUIRED` `username`: Username for authenticating with the specified SMTP server.
- `REQUIRED` `password`: Password for authenticating with the specified SMTP server.
- `REQUIRED` `from`: Email address where emails are sent from.
- `REQUIRED` `reply_to`: Email address to which replies on the emails should be sent. Can also be a non reply address, or the same as the from address.

### `REQUIRED` `[auth]` Section
- `REQUIRED` `min_password_length`: Minimum length of a password when registering a new user.
- `REQUIRED` `max_password_length`: Maximum length of a password when registering a new user.
- `REQUIRED` `secret_key`: Signing key of the JWT authentication tokens. Keeping these default will severly impact the security of your instance, and allows attackers to login as any user.

### `REQUIRED` `[storage]` Section
- `REQUIRED` `upload_path`: Path where uploads should be stored. Directories will be automatically created on startup if they don't exist.

## Default Configuration
```toml
[tracker]
# tracker UDP url with PORT.
url = "udp://localhost:6969"
# tracker REST API url with PORT.
api_url = "http://localhost:1212"
# token needs to be set to use the tracker REST API.
token = "MyAccessToken"
# 12 weeks
token_valid_seconds = 7257600

[net]
port = 3000

[database]
# mode rwc means read/write/create.
# without create, you'll have to manually create the database.
connect_url = "sqlite://data.db?mode=rwc"
# recommended to keep it at least at 1800 seconds.
# interval should increase with amount of torrents.
torrent_info_update_interval = 3600

[mail]
# SMTP server and port
server = ""
port = 25
# username and password for authenticating with the SMTP server
username = ""
password = ""
# email address where mail is sent from
from = ""
# email address to which a reply can be sent
reply_to = ""

[auth]
min_password_length = 6
max_password_length = 64
# IMPORTANT: change to some random chars.
# DO NOT KEEP THIS DEFAULT SECRET.
secret_key = "MaxVerstappenWC2021"

[storage]
# storage path for uploaded torrent files.
upload_path = "./uploads"
```
