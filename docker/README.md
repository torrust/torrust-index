# Docker

## Requirements

- Docker version 20.10.21
- You need to create the `storage` directory with this structure and files:

```s
$ tree storage/
storage/
└── database
    ├── data.db
    └── tracker.db
```

## Dev environment

### With docker

Build and run locally:

```s
docker context use default
export TORRUST_IDX_BACK_USER_UID=1000
./docker/bin/build.sh $TORRUST_IDX_BACK_USER_UID
./bin/install.sh
./docker/bin/run.sh $TORRUST_IDX_BACK_USER_UID
```

Run using the pre-built public docker image:

```s
export TORRUST_IDX_BACK_USER_UID=1000
docker run -it \
    --user="$TORRUST_IDX_BACK_USER_UID" \
    --publish 3000:3000/tcp \
    --volume "$(pwd)/storage":"/app/storage" \
    torrust/index-backend
```

> NOTES:
>
> - You have to create the SQLite DB (`data.db`) and configuration (`config.toml`) before running the index backend. See `bin/install.sh`.
> - You have to replace the user UID (`1000`) with yours.
> - Remember to switch to your default docker context `docker context use default`.

### With docker-compose

The docker-compose configuration includes the MySQL service configuration. If you want to use MySQL instead of SQLite you have to change your `config.toml` or `config-idx-back.toml.local` configuration from:

```toml
connect_url = "sqlite://storage/database/data.db?mode=rwc"
```

to:

```toml
connect_url = "mysql://root:root_secret_password@mysql:3306/torrust_index_backend"
```

If you want to inject an environment variable into docker-compose you can use the file `.env`. There is a template `.env.local`.

Build and run it locally:

```s
TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000} \
    TORRUST_IDX_BACK_CONFIG=$(cat config-idx-back.toml.local) \
    TORRUST_TRACKER_CONFIG=$(cat config-tracker.toml.local) \
    TORRUST_TRACKER_API_TOKEN=${TORRUST_TRACKER_API_TOKEN:-MyAccessToken} \
    docker compose up -d --build
```

After running the "up" command you will have three running containers:

```s
$ docker ps
CONTAINER ID   IMAGE                     COMMAND                  CREATED          STATUS                    PORTS                                                                                            NAMES
e35b14edaceb   torrust-idx-back          "cargo run"              19 seconds ago   Up 17 seconds             0.0.0.0:3000->3000/tcp, :::3000->3000/tcp                                                        torrust-idx-back-1
ddbad9fb496a   torrust/tracker:develop   "/app/torrust-tracker"   19 seconds ago   Up 18 seconds             0.0.0.0:1212->1212/tcp, :::1212->1212/tcp, 0.0.0.0:6969->6969/udp, :::6969->6969/udp, 7070/tcp   torrust-tracker-1
f1d991d62170   mysql:8.0                 "docker-entrypoint.s…"   3 hours ago      Up 18 seconds (healthy)   0.0.0.0:3306->3306/tcp, :::3306->3306/tcp, 33060/tcp                                             torrust-mysql-1
                                                                             torrust-mysql-1
```

And you should be able to use the application, for example making a request to the API:

<http://localhost:3030>

The Tracker API is available at:

<http://localhost:1212/api/v1/stats?token=MyAccessToken>

> NOTICE: You have to bind the tracker services to the wildcard IP `0.0.0.0` to make it accessible from the host.

You can stop the containers with:

```s
docker compose down
```

Additionally, you can delete all resources (containers, volumes, networks) with:

```s
docker compose down -v
```

### Access Mysql with docker

These are some useful commands for MySQL.

Open a shell in the MySQL container using docker or docker-compose.

```s
docker exec -it torrust-mysql-1 /bin/bash 
docker compose exec mysql /bin/bash
```

Connect to MySQL from inside the MySQL container or from the host:

```s
mysql -h127.0.0.1 -uroot -proot_secret_password
```

The when MySQL container is started the first time, it creates the database, user, and permissions needed.
If you see the error "Host is not allowed to connect to this MySQL server" you can check that users have the right permissions in the database. Make sure the user `root` and `db_user` can connect from any host (`%`).

```s
mysql> SELECT host, user FROM mysql.user;
+-----------+------------------+
| host      | user             |
+-----------+------------------+
| %         | db_user          |
| %         | root             |
| localhost | mysql.infoschema |
| localhost | mysql.session    |
| localhost | mysql.sys        |
| localhost | root             |
+-----------+------------------+
6 rows in set (0.00 sec)
```

```s
mysql> show databases;
+-----------------------+
| Database              |
+-----------------------+
| information_schema    |
| mysql                 |
| performance_schema    |
| sys                   |
| torrust_index_backend |
| torrust_tracker       |
+-----------------------+
6 rows in set (0,00 sec)
```

If the database, user or permissions are not created the reason could be the MySQL container volume can be corrupted. Delete it and start again the containers.
