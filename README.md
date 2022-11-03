# ch559 flasher

## Setup
```
$ cargo install --path .
```

## Usage
```
$ ch559flasher -h
CH559 flash utility

Usage: ch559flasher [OPTIONS] [FILENAME]

Arguments:
  [FILENAME]  Filename to flash from or write into

Options:
  -e, --erase         Erase program area
  -w, --write         Write FILENAME to program area
  -c, --compare       Compare program area with FILENAME
  -E, --erase-data    Erase data area
  -R, --read-data     Read data area to FILENAME
  -W, --write-data    Write FILENAME to data area
  -C, --compare-data  Compare data area with FILENAME
  -f, --fullfill      Fullfill unused area with randomized values
  -s, --seed <SEED>   Random seed
  -h, --help          Print help information
  -V, --version       Print version information
```

## Examples

### Program and verify
```
$ ch559flasher -w -c firmware.bin
CH559 Found (BootLoader: v2.31)
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
$ ch559flasher -w -c -f firmware.bin
CH559 Found (BootLoader: v2.31)
[##################################################] (61440 bytes)
write: complete
[##################################################] (61440 bytes)
compare: complete
```