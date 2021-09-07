# Torrust REST API

## User endpoints (v1)
### `POST /user/register`
Register (sign-up) a new user account.

`Example: POST /user/register`
#### Body:
```json 
{
	"email": "email@example.com",
	"username": "example_user",
	"password": "password",
	"confirm_password": "password"
}
```

#### Response:
- On success: Status `200`, with an empty body.
- On error: Standard error, see [Errors](API.md#errors)

---

<a id="login"></a>
### `POST /user/login`
Login into an existing account.

#### Fields
- `login`: Either the email or username of the account
- `password`: password of the account
<br><br>

`Example: POST /user/login`
#### Body:
```json 
{
	"login": "email@example.com",
	"password": "password"
}
```

#### Response:
- On success: Status `200`.
```json
{
  "data": {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJleGFtcGxlX3VzZXIiLCJleHAiOjE2MzIyNTQxNjZ9.kyugZXiR88q4n6Ze44HonazDp-sJdq886te9-XHthHg",
    "username": "example_user"
  }
}
```
- On error: Standard error, see [Errors](API.md#errors)

---

### `GET /user/verify/<token>`
Email verification handler. <br>
On register an email is sent to the email address of the registered account with a link to this route to verify their email address.

![email verification mail](images/email-verification.png)

`Example: GET /user/verify/<token>`
#### Response:
- On success: Status `200`.
```html
Email verified, you can close this page.
```
- On error: Error message as a string.
```html
Token invalid.
```

---