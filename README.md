# mk

Text preprocessor for content segmentation.

## Examples

### Basic

`mkfile` file

```shell
TARGET="mk"
build:
  go build -o ./bin/${TARGET}

clean:
  go clean
  rm -f ./bin/${TARGET}
```

`mk build` output

```shell
TARGET="mk"
go build -o ./bin/${TARGET}
```

`mk clean` output

```shell
TARGET="mk"
go clean
rm -f ./bin/${TARGET}
```

### Shebang

`mkfile` file

```shell
#!/usr/bin/bash
echo "zero"
one:
  echo "one"
echo $1
two:
  echo "two"
```

`mk one` output

```shell
zero
one
```

`mk two my-one` output

```shell
zero
my-one
two
```

## Installation

### Via `go install`

With go 1.18 or higher:

```shell
go install github.com/x0k/mk@latest
```

## Explanation

- The segment defined by a label that satisfies this regular expression `^[A-Za-z][0-9A-Za-z\t_-]*:$` and by the presence of equal indentation `^([ \t]+)` on the subsequent lines.

  By default segment is not defined.

- The end of a segment is determined by indentation changes or the end of the file.
- If segment is not defined for a line, the line will be added to each segment defined below.
- Allowed file names `mkfile`, `Mkfile`
