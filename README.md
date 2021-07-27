## Summary

Sample code for upload or download a file from Azure Storage, using [azure-sdk-for-rust](https://github.com/Azure/azure-sdk-for-rust) crate.

## How to Build

We have checked with the following toolchain versions.

- cargo 1.55.0-nightly (3ebb5f15a 2021-07-02)
- rustc 1.55.0-nightly (952fdf2a1 2021-07-05)


## How to Use

```
azure-storage 0.1.0
ADVALY SYSTEM Inc.
Azure Storage file uploader and downloader

USAGE:
    azure-storage [FLAGS] [OPTIONS] <list|get|put|append|put-append|delete>

FLAGS:
        --debug      Enable debug print
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --blob <blob>                                Remote blob name on Azure Storage
        --config <config>                            Config file path [default: azure-storage.json]
    -c, --container <container>                      Remote container name on Azure Storage
    -l, --local <local>                              Local file path to put or get
    -a, --storage_account <storage account>          STORAGE_ACCOUNT
    -k, --storage_master_key <storage master key>    STORAGE_MASTER_KEY

ARGS:
    <list>          List objects on remote
    <get>           Get a blob from remote
    <put>           Put a block blob to remote
    <append>        Append a file to existing append blob
    <put-append>    Create a new append blob to remote
    <delete>        Delete a blob from remote
```

### Set Azure Storage Accounts

There are two ways to set access accounts.
(One more thing...configuration file described later)

#### Environment variables

Set `STORAGE_ACCOUNT` and `STORAGE_MASTER_KEY` as environment variables.

Example:
```
$ export STORAGE_ACCOUNT=id
$ export STORAGE_MASTER_KEY=key
```

#### Pass as command line arguments

Example:
```
$ azure-storage list --storage_account=<id> --storage_master_key=<key> ...

shorter expression:
$ azure-storage list -a<id> -k<key> ...
```

### Operation examples

In the following examples, access accounts are assumed to be set as envirinment variables.

#### LIST

Example1: Show list of containers
```
$ azure-storage list
```

Example2: Show list of blobs in a specified container
```
$ azure-storage list --container=test

shorter expression:
$ azure-storage list -ctest
```

#### GET

Get a file from Azure Strage.

Need to specify local path, container name and blob name with command line arguments.

- `--container`: Target container
- `--blob`: Target blob to get from the Azure Storage
- `--local`: Local path to save the retrieved file
  - If you specify a directory path for `local`, the destination local file name is set to the same name as remote blob

Example1: Specify directory path for `local`. 'hoge.txt' on the Azure Storage is retrieved as '/tmp/hoge.txt'.
```
$ azure-storage get --container=test --blob=hoge.txt --local=/tmp

shorter expression:
$ azure-storage get -ctest -bhoge.txt -l/tmp
```

Example2: Speficy absolute path for `local`. 'hoge.txt' on the Azure Storage is retrieved as '/tmp/fuga.txt'
```
$ azure-storage get --container=test --blob=hoge.txt --local=/tmp/fuga.txt

shorter expression:
$ azure-storage get -ctest -bhoge.txt -l/tmp/fuga.txt
```

#### PUT

Put a file to Azure Strage.

Need to specify local path, container name and blob name (optional) with command line arguments.

- `--local`: Local path for a file to put
- `--container`: Target container
- `--blob` (optional): Target blob name to put on the Azure Storage
  - If you ommited `blob`, the destination blob name is set to the same name as local file name

Example1:
```
$ azure-storage put --container=test --blob=piyo.txt --local=hoge.txt

shorter expression:
$ azure-storage put -ctest -bpiyo.txt -lhoge.txt
```

Example2:
```
$ azure-storage put --container=test --local=/tmp/hoge.txt

shorter expression:
$ azure-storage put -ctest -l/tmp/hoge.txt
```

#### APPEND

Append a file to an append blob on Azure Strage.
The target blob must already exist as blob type of 'Append Blob'.

The other is same as `put` operation.

Example:
```
$ azure-storage append --container=test --blob=piyo.txt --local=hoge.txt

shorter expression:
$ azure-storage append -ctest -bpiyo.txt -lhoge.txt
```

#### PUT-APPEND

Create a new append blob on Azure Storage. This operation does just create a new empty blob.

Need to specify container name and blob name with command line arguments.

- `--container`: Target container
- `--blob`: Target blob name to create on the Azure Storage

Example:
```
$ azure-storage put-append --container=test --blob=piyo.txt

shorter expression:
$ azure-storage put-append -ctest -bpiyo.txt
```

#### DELETE

Delete a file from Azure Storage.

Need to specify a container name and blob name to delete with command line arguments.

- `--container`: Target container
- `--blob`: Target blob to delete from the Azure Storage

Example:
```
$ azure-storage delete --container=test --blob=fuga.txt

shorter expression:
$ azure-storage delete -ctest -bfuga.txt
```

## Configuration File

You can also use a configuration file to abbreviate command line arguments.

Following parameters can be load from a configuration file instead of specifying on the command line.
The default configuration file name is 'azure-storage.json' in the current directory.
This default file name is changed by the command line option `--config`.

When the configuration file found, azure-storage load followig settings from the configuration file.

- storage account
- storage master key
- local

If same parameters are speficied by command line even though the configuration file is loaded,
azure-storage uses command line arguments first.

The Azure access keys also could be defined as environment variable. So the priorities are as follows.

Command line options > Configuration file > Environment variables

### File format

The configuration file is described in json format.

Example: azure-storage.json
```json
{
    "storage_account": "your storage account id",
    "storage_master_key": "your storage master key",
    "local": "/tmp"
}
```

You do not need to fill all the value in the configuration file.

For example if you want to set only `storage_account` and `storage_master_key` parameters in the configuration file, you do not need to write definitions of `local`. Leave as blank string "". 
