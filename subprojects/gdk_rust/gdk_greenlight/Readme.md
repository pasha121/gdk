# Test

command launched from `subprojects/gdk_rust` dir

## Unit test

```shell
cargo test -p gdk-greenlight
```

Without `WALLY_DIR` otherwise multiple symbols are found.


## Integration test

### Run the backend

In a dedicated shell, start the docker (view details https://github.com/Blockstream/greenlight/blob/4a24b16066f73cfde94cd06eaf17d6f19f2d51e0/libs/gl-testing/README.md) by launching the following bash script:

```
docker run -ti --rm -v ${PWD}/gdk_greenlight:/repo  --net host gltesting pytest -s test_regtest.py
```

this will create files under the directory `gdk_greenlight/regtest` that are removed once you stop the backend by using "Ctrl-c".
If the docker close for other reasons, directory `gdk_greenlight/regtest` needs manual removal.

### Run the tests

In the shell where rust tests are launched, launch the shell script:

```bash
. ./gdk_greenlight/backend_env.sh
```

Note that if you re-launch the docker backend addresses will change so you need to re-launch the script.
Note also at the moment a `socat` process is spawned and need to be manually killed (eg. `killall socat`)

then use 

```bash
cargo test -p gdk-greenlight -- --include-ignored --test-threads 1
```

Note you can connect to inner non-GL lightning node using `lightning-cli`

```bash
lightning-cli --rpc-file /tmp/unix-sock-${GL_L1_PORT} getinfo
```

### View pb structs

```bash
cd greenlight/libs/gl-client
cargo +nightly expand --color always pb | less -r
```

### Update greenlight repo

At the moment greenlight client libs are not published and neither public.
In order to work we include the files directly with the following commands.

```sh
rm -rf $PATH_TO_GDK_RUST/greenlight
cd /tmp
git clone git@github.com:Blockstream/greenlight.git
cd greenlight
git checkout $BRANCH
git submodule update --init 
rm -rf .git
cd $PATH_TO_GDK_RUST
mv /tmp/greenlight .
```

Then remove the line with field `preimage` in `InvoiceRequest` in `gdk_rust/greenlight/libs/proto/greenlight.proto` and fix errors in `pb.rs`.
Preimage field will become optional once migrated to glrpc