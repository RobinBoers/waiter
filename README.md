# Waiter

A simple HTTP gridserver built for serving gridsites. It handles caching and mime types out of the box and supports publishing using the HTTP `PUT` verb. And it is FAST.

## Features

- Automatic cache control
- Serves `.htmd` files as `text/plain` on unsupported browsers
- Stable and fast
- Easy to use
- Supports `PUT` for publishing files
  - Requires auth (HTTP Basic Auth) for `PUT` requests

Upcoming:

- Fancy URLs for `.htmd` files

## Usage

Build the binary and run it in the directory containing the files to be served.
