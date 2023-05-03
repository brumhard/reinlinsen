# Tasks

## test-run

> builds the test docker image and runs

```sh
image_name="reinlinsen-test"
rm -rf "$image_name"*
docker build -f test.Dockerfile  -t "$image_name" .
cargo run -- --image "$image_name" dump
```
