# reinlinsen 👀

`rl` is a tool to inspect and dump OCI images or single image layers.

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

## Troubleshooting

- <https://github.com/cross-rs/cross/issues/1184>
- "version `GLIBC_2.28' not found" -> <https://github.com/cross-rs/cross/issues/724>
