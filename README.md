# TritonFile
Final Project: Distributed file system based on FUSE

## Dependencies

https://github.com/cberner/fuser

We follow the instructions here to install dependencies for macOS.

## Launch the File System

Find a directory to use as your working directory, then run:

```
cargo run --bin bins-mkcfg -- --backs 3
cargo run --bin bins-back
cargo run --bin bins-keep
```

This will generate a file called `bins.json` under the current directory, which contains the configuration of backends and keepers. 



To launch the file system, we should first modify the mountpoint parameter defined in`cmd/src/trib_front.rs` fuser::mount2, and then run:

```
cargo run --bin trib-front
```

Then we can execute the following commands under :

```
touch test.txt
echo "hello" > test.txt
cat test.txt
cp test.txt test2.txt
rm test.txt
```

