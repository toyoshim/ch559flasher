# ch559 flasher

## Setup
```
$ cargo install --path .
```

## Usage
```
$ ch559flasher -h
CH559 flash utility

Usage: ch559flasher [OPTIONS]

Options:
  -e, --erase                              Erase program area
  -w, --write-program <WRITE_PROGRAM>      Write a specified file to program area
  -c, --compare-program <COMPARE_PROGRAM>  Compare program area with a specified file
  -E, --erase-data                         Erase data area
  -R, --read-data <READ_DATA>              Read data area to a specified file
  -W, --write-data <WRITE_DATA>            Write a specified file to data area
  -C, --compare-data <COMPARE_DATA>        Compare data area with a specified file
  -f, --fullfill                           Fullfill unused area with randomized values
  -s, --seed <SEED>                        Random seed
  -g, --config <CONFIG>                    Write BOOT_CFG[15:8] in hex (i.e. 4e)
  -b, --boot                               Boot application
  -h, --help                               Print help
  -V, --version                            Print version
```

## Examples

### Program and verify
```
$ ch559flasher -w firmware.bin -c firmware.bin
CH559 Found (BootLoader: v2.31)
erase: complete
[##################################################] (59293 bytes)
write: complete
[##################################################] (59293 bytes)
compare: complete
```

### Read data area into a file
```
$ ch559flasher -R data.bin
CH559 Found (BootLoader: v2.31)
[##################################################] (1024 bytes)
read_data: complete
```

### Clear code and data
```
$ ch559flasher -e -E
CH559 Found (BootLoader: v2.31)
erase: complete
erase_data: complete
```

### Program and verify (with fullfilling unused area with random values)
```
$ ch559flasher -w firmware.bin -c firmware.bin -f
CH559 Found (BootLoader: v2.31)
erase: complete
[##################################################] (61440 bytes)
write: complete
[##################################################] (61440 bytes)
compare: complete
```