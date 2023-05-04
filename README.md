# reinlinsen

`rl` is a tool to inspect OCI images.

goal:

```shell
rl layer ls <image> -> list layers with creation command
rl layer inspect <image> -l <layer> -> show layer info with included files
rl layer dump <image> -l <layer> -o dir -> dump only this layer
rl layer dump <image> -l <layer> -o dir --stack -> include preceding layers into the output
rl layer extract <image> -l <layer> -f file -o file -> extract a file from the layer
rl dump <image> -o dir -> full dump of all layers
rl extract <image> -f file -o file -> extract a file from the full dump
```