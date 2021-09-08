# Torrust

[![Build Status](https://app.travis-ci.com/torrust/torrust.svg?branch=master)](https://app.travis-ci.com/torrust/torrust)

Torrust is an open source web based BitTorrent tracker developed in Rust.
It allows users to upload and download torrents on a web UI, and tracks peers with an UDP BitTorrent tracker.

## Project structure
- [__Torrust__](https://github.com/torrust/torrust) (This repo): A REST API that acts as a backend for Torrust Web.
- [__Torrust Web__](https://github.com/torrust/torrust-web): A Vue application where torrents can be uploaded and downloaded.
- [__Torrust Tracker__](https://github.com/torrust/torrust-tracker): A UDP based torrent tracker built with Rust.

## Features
* [X] Login / Register
* [X] Authentication using JWT tokens
* [X] E-mail verification
* [X] Torrent Uploading / Downloading

## Contributing
Please report any bugs you find to our issue tracker. Ideas and feature requests are welcome as well!
Any pull request targeting existing issues would be very much appreciated.
