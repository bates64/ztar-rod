This directory contains files used to steal assets from Star Rod.

It is a temporary measure used to bootstrap ztar rod tools and should never
appear in mainline `ztar-rod`.

# Usage

Copy `StarRod.jar` into `steal`.

```sh
$ cd steal
$ javac -cp patch:StarRod.jar patch/**.java

$ java -cp patch:StarRod.jar DumpMap .../dump/map/src/arn_bt02.map > arn_bt02.json
# produces steal/arn_bt02.json
```

Be careful not to mix up assets dumped from Star Rod with those dumped from
`ztar-rod`. You should put stolen assets in `steal/...`, where `ztar-rod` will
look to process them further.

It is possible to use the rest of the patches to launch Star Rod on macOS but
that is out of scope for this document.
