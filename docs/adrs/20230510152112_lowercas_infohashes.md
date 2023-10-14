# Lowercase infohashes

## Description

We use both uppercase and lowercase infohashes. This is a problem because
we have to check both cases. For example, we have to convert to uppercase before
inserting into the database or querying the database.

The database and API URLs use uppercase infohashes, and they are case-sensitive.

## Agreement

We agree on use lowercase infohashes everywhere and try to convert then as soon
as possible from the input.

There is no specific reason to use lowercase infohashes, but we have to choose
one of them. We decided to use lowercase because the infohash is a hash, and
hashes are usually lowercase.

We will change them progressively.
