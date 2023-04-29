# Waiter

A simple HTTP gridserver built for serving gridsites. It handles caching and mime types out of the box and supports publishing using the HTTP `PUT` verb. And it is FAST.

## Features

- Automatic cache control
- Serves `.htmd` files as `text/plain` on unsupported browsers
- Stable and fast
- Easy to use
- Fancy URLs
- Supports `PUT` for publishing files
  - Requires auth (HTTP Basic Auth) for `PUT` requests

> Note:  
> Uploads are currently broken, because path canonicalization in Rust apparently fails if the path doesn't exist yet (while I expected it to just remove the `../` things from the path, even if the final path doesn't exist). Fixing this is a huge refactor, which I can't be bothered to do. Overwriting existing files does work (since the path already exists then), so yay!

## Usage

Build the binary and run it in the directory containing the files to be served.
