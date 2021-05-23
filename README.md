# rune-serve
Simple HTTP/Ngrok Serve to quick serve your Rune

## How to use

``` mkdir static ```

Write your Runefile in the static folder.

``` rune build Runefile ```

``` docker run -e RUST_LOG=INFO -v `pwd`/static:/app/static tinyverseml/rune-serve ```
