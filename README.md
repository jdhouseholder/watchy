# watchy 🐋

Watch files and runs a command on file change with the file path passed as an argument.

## Usage

```
watchy --watch ./foo.txt --then bar.sh
```

When `./foo.txt` changes watchy will execute `bar.sh ./foo.txt`.

## Notes

We use IN\_ONESHOT because some programs update by moving.
