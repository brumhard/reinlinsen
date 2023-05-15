# reinlinsen 👀

`rl` is a tool to inspect and dump OCI images or single image layers.

<details>
<summary><h2>Installation</h3></summary>

### From source

If you have `cargo` installed you can just run the following.
Make sure that you have added Cargo's bin directory (e.g. `~/.cargo/bin`) to your `PATH`.

```shell
cargo install --git https://github.com/brumhard/reinlinsen.git --tag latest
```

### Released binaries/packages

Download the desired version for your operating system and processor architecture from the [releases](https://github.com/brumhard/reinlinsen/releases).
Make the file executable and place it in a directory available in your `$PATH`.

### Use with nix

```shell
nix run github:brumhard/reinlinsen/latest
```

or

```nix
{
    inputs.reinlinsen.url = "github:brumhard/reinlinsen/latest";

    outputs = { reinlinsen, ... }: {
        packages.x86_64-linux = [reinlinsen.packages.x86_64-linux.rl];
    };
}
```

### Homebrew

```shell
brew install brumhard/tap/reinlinsen
```

</details>

## Features

```shell
rl dump <image> -o <dir> # full dump of all layers
rl extract <image> -p <src> -o <dest> # extract a file or dir from the full dump

rl layer ls <image> # list all image layers
rl layer inspect <image> -l <layer> # show image layer's files
rl layer dump <image> -l <layer> -o <dir> # dump only this layer
rl layer dump <image> -l <layer> -o <dir> --stack # include preceding layers into the output
rl layer extract <image> -l <layer> -p <src> -o <dest> # extract a file or dir from the layer
```

## Example

```shell
$ docker pull alpine
Using default tag: latest
latest: Pulling from library/alpine
08409d417260: Pull complete 
Digest: sha256:02bb6f428431fbc2809c5d1b41eab5a68350194fb508869a33cb1af4444c9b11
Status: Downloaded newer image for alpine:latest
docker.io/library/alpine:latest

$ rl dump alpine -o alpine

$ tree -L 1 alpine 
alpine
├── bin
├── dev
├── etc
├── home
├── lib
├── media
├── mnt
├── opt
├── proc
├── root
├── run
├── sbin
├── srv
├── sys
├── tmp
├── usr
└── var

$ file alpine/bin/busybox 
alpine/bin/busybox: ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV), dynamically linked, interpreter /lib/ld-musl-aarch64.so.1, stripped
```
