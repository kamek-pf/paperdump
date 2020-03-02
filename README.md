# paperdump
Deal with paperwork quickly so you can move on to something more interesting.

## What ?
Like everyone else, you probably have to deal with some paperwork that inevitably comes in the mail. \
When you're done, you're supposed to store it somewhere, "just in case". This is the cumbersome part.

Where do you store these documents ? How do you organize them ? You need to be able to find them afterwards, somehow ...

So here's the idea, when a bill (or whatever else) comes in, deal with it, then:
- Scan the document, have the scanner send it to your FTP server
- Stamp the document with today's date
- Shove it in a folder

And that's it. When you're done with your bills, just stack them in a folder, in the order you receive them. \
In the background, `paperdump` watches your filesystem. When a document comes in it will:
- Move it to a permanent directory
- Run [Tesseract](https://en.wikipedia.org/wiki/Tesseract) on the scan, which gets you a plain text file.
- [TODO]: Compute a small preview

From now on, if you ever need to find the original document, just use can use [`ripgrep`](https://github.com/BurntSushi/ripgrep)
(or any other full text search tool) with a keyword, and you'll know the exact date you scanned the document you're looking for.

## Dependencies
You'll need `clang`, `leptonica` and `tesseract` with support for your language to build and run the project. On arch:
```
pacman -S clang leptonica tesseract tesseract-data-eng
```

## Build
`cargo build --release`, you'll need a working Rust toolchain to build this project. \
The build will be available at `target/release/paperdump`. \
Check out `rustup` to get up and running quickly.

## Usage
Run `paperdump -c config.toml`, let it run in the background. \
Files uploaded to the `watch_folder` will be processed and moved to the `destination_folder`.

## Credits
Credits to my coworker Lewis for the cool idea !