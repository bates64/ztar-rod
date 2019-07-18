This directory contains files used to steal assets from Star Rod.

It is a temporary measure used to bootstrap ztar rod tools and should never
appear in mainline `ztar-rod`.

# Usage

Copy `StarRod.jar` into `steal`. Also copy the Star Rod asset directories
`image` and `map`.

```sh
$ cd steal
$ javac -cp patch:StarRod.jar patch/**.java
$ java -cp patch:StarRod.jar MapDumper
```

Be careful not to mix up assets dumped from Star Rod with those dumped from
`ztar-rod`. You should put stolen assets in `steal/...`, where `ztar-rod` will
look to process them further.

It is possible to use the rest of the patches to launch Star Rod on macOS but
that is out of scope for this document.
