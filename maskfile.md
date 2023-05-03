# Tasks

## test-run

> builds the test docker image and runs dump

```sh
image_name="reinlinsen-test"
out_dir="$image_name-fs"
rm -rf "$out_dir"
docker build -f test.Dockerfile  -t "$image_name" .
cargo run -- --image "$image_name" dump -o "$out_dir" --verbose
```
