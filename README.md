# reinlinsen

`rl` is a tool to inspect OCI images.

goal:

```shell
rl layer ls <image> -> list layers with creation command
rl layer inspect <image> -l <layer> -> show layer info with included files
rl layer dump <image> -l <layer> -o dir -> dump only this layer
rl layer dump <image> -l <layer> -o dir --stack -> include preceding layers into the output
rl dump <image> -o dir -> full dump of all layers
```
