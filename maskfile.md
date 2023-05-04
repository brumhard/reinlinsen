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

## cross

> builds the images for all supported platforms

**OPTIONS**

* filter
  * flags: --filter -f
  * type: string
  * desc: filter all targets with the given string, e.g. "linux", "aarch64"

```sh
out_dir="out"
targets=$(yq -o json -p toml -r '.toolchain.targets[]' rust-toolchain)
if [ "$filter" != "" ]; then
    targets=$(echo "$targets"|rg "$filter")
fi
mkdir -p "$out_dir"/bin
for target in $targets; do
    echo "building for $target"
    # specifying target-dir is a hack for https://github.com/cross-rs/cross/issues/724
    cross build --release --target "$target" --target-dir "$out_dir/$target"
    cp "$out_dir/$target/$target/release/rl" "$out_dir/bin/rl-$target"
done
```
