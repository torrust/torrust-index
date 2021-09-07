# Torrust REST API

## Category endpoints (v1)
### `GET /`
Get a list of existing categories.

#### Response:
- On success: Status `200`.
```json
{
  "data": [
    {
      "name": "app",
      "num_torrents": 0
    },
    {
      "name": "documentary",
      "num_torrents": 0
    },
    {
      "name": "game",
      "num_torrents": 0
    },
    {
      "name": "movie",
      "num_torrents": 1
    },
    {
      "name": "music",
      "num_torrents": 0
    },
    {
      "name": "other",
      "num_torrents": 0
    },
    {
      "name": "tv show",
      "num_torrents": 0
    }
  ]
}
```
- On error: Standard error, see [Errors](API.md#errors)

---

### `GET /category/<category>/torrents`
Get an array of torrents that fall under the specified category.

#### Response:
- On success: Status `200`.
```json
{
  "data": {
    "total": 2,
    "results": [
      {
        "torrent_id": 2,
        "uploader": "example_user",
        "info_hash": "73844d3f5d163a6a778920e84aa084cc0746cd72",
        "title": "Movie torrent",
        "description": "## Awesome Movie 4\n\nAwesome Movie 4 is set in Alabama and is the succesor of Awesome Movie 3.\nGet ready for an even bigger adventure.",
        "category_id": 1,
        "upload_date": 1631047197,
        "file_size": 1243947,
        "seeders": 0,
        "leechers": 0
      },
      {
        "torrent_id": 1,
        "uploader": "example_user",
        "info_hash": "5499b9f42b44fb61c937be5943a194adb7aa6278",
        "title": "Example torrent",
        "description": "## Some torrent title\n\nSome torrent text.\n\n---",
        "category_id": 1,
        "upload_date": 1631046870,
        "file_size": 15653809,
        "seeders": 0,
        "leechers": 0
      }
    ]
  }
}
```
- On error: Standard error, see [Errors](API.md#errors)

---