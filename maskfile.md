# Tasks

## init

> initializes the dev env in the repo

```bash
git config --local core.hooksPath .githooks/
```

## test-run

> builds the test docker image and runs dump

```bash
image_name="reinlinsen-test"
out_dir="$image_name-fs"
rm -rf "$out_dir"
docker build -f test.Dockerfile  -t "$image_name" .
cargo run -- dump "$image_name" -o "$out_dir" --verbose
```

## lint

> runs cargo clippy

```bash
cargo clippy -- -W clippy::pedantic
```

## audit

> runs audit for dependencies

```bash
info=$(cargo outdated --root-deps-only --format json)
if [ $(echo "$info" |  jq '.dependencies | length') -gt 0 ]; then
    echo "dependencies are not up to date:"
    echo "$info" | jq
    exit 1
fi
vulns=$(cargo audit --json)
if [ $(echo "$vulns" |  jq '.vulnerabilities.count') -gt 0 ]; then
    echo "vulnerabilities found:"
    echo "$vulns" | jq
    exit 1
fi
```

## build

> builds the images for (possibly filtered) targets

**OPTIONS**

* filter
  * flags: --filter -f
  * type: string
  * desc: filter all targets with the given string, e.g. "linux", "aarch64"

```bash
set -eo pipefail
targets=$(yq -o json -p toml -r '.toolchain.targets[]' rust-toolchain.toml)
if [ "$filter" != "" ]; then
    targets=$(echo "$targets" | rg "$filter")
fi

out_dir="out"
rm -rf "$out_dir"/bin
mkdir -p "$out_dir"/bin

build_args=""
if [ $verbose ]; then
    build_args="--verbose"
fi

for target in $targets; do
    echo "building for $target"
    # specifying target-dir is a hack for https://github.com/cross-rs/cross/issues/724
    cross build --release --target "$target" --target-dir "$out_dir/$target" $build_args
    arch_os=$(echo "$target" | rg '^(?P<arch>.+?)-\w+-(?P<os>\w+)(-\w*)?$' -r '$arch-$os')
    cp "$out_dir/$target/$target/release/rl" "$out_dir/bin/rl-$arch_os"
done
```

## tag

> creates a new tag

**OPTIONS**

* next_tag
  * flags: --tag -t
  * desc: tag for the next release version
  * type: string

```bash
set -eo pipefail
if [ "$(git status --porcelain)" != "" ]; then
    echo "nope too dirty"
    exit 1
fi
if [ "$next_tag" = "" ]; then
    current_tag=$(git tag |tail -1)
    proposed_tag=$(svu n)
    read -r -p "Enter next tag or accept proposed (current: '$current_tag', proposed: '$proposed_tag'): " next_tag 
    if [ "$next_tag" = "" ]; then
        next_tag="$proposed_tag"
    fi
fi
# check valid version
if ! echo "$next_tag" | rg -q 'v([0-9]|[1-9][0-9]*)\.([0-9]|[1-9][0-9]*)\.([0-9]|[1-9][0-9]*)'; then
    echo "not a valid version"
    exit 1
fi
# set version without leading v
cargo set-version "${next_tag:1}"
git add Cargo.*
git commit --no-verify --message "chore: bump package to $next_tag"
git tag "$next_tag"
git push --no-verify
git push --no-verify --tags
```

## test-release

> creates a new release snapshot

```bash
$MASK build --filter darwin
goreleaser release --snapshot --skip-validate --clean
```
