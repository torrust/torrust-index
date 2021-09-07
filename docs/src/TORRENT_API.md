# Torrust REST API

## Torrent endpoints (v1)
### `POST /torrent/upload`
Upload a torrent file and create a torrent listing for it in the index.

Consumes a __mutipart__ form with the fields:
- `title`: Title of the torrent listing.
- `description`: A Markdown description.
- `category`: Category this torrent fits in.
- `torrent`: The torrent file itself.

#### Response:
- On success: Status `200`, returns id of the newly created torrent listing.
```json
{
    "data": {
        "torrent_id": 1
    }
}
```
- On error: Standard error, see [Errors](API.md#errors)

---

### `GET /torrent/download/<id>`
Generate and download torrent file with a personal annouce URL for the authenticated user.

#### Response:
- On success: Status `200`, Personalised .torrent file stream.
- On error: Standard error, see [Errors](API.md#errors)

---

### `GET /torrent/<id>`
Get all torrent information of the listing with `id`, used for loading the data of the TorrentDetails page.

#### Response:
- On success: Status `200`.
```json
{
  "data": {
    "torrent_id": 1,
    "uploader": "example_user",
    "info_hash": "5499b9f42b44fb61c937be5943a194adb7aa6278",
    "title": "Example torrent",
    "description": "## Some torrent title\n\nSome torrent text.\n\n---",
    "category_id": 1,
    "upload_date": 1631046870,
    "file_size": 15653809,
    "seeders": 0,
    "leechers": 0,
    "files": null
  }
}
```
- On error: Standard error, see [Errors](API.md#errors)

---