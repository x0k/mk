# Cook

Text preprocessor for content segmentation.

## Examples

### Basic

`receipts` file

```shell
TARGET="cook"
build:
  go build -o ./bin/${TARGET}

clean:
  go clean
  rm -f ./bin/${TARGET}
```

`cook build` output

```shell
TARGET="cook"
go build -o ./bin/${TARGET}
```

`cook clean` output

```shell
TARGET="cook"
go clean
rm -f ./bin/${TARGET}
```

### Shebang

`receipts` file

```shell
#!/usr/bin/bash
echo "zero"
one:
  echo "one"
echo "two"
three:
  echo "three"
```

`cook one` output

```shell
zero
one
```

`cook three` output

```shell
zero
two
three
```

## Installation

### Via `go install`

With go 1.18 or higher:

```shell
go install github.com/x0k/cook@latest
```

## Explanation

- The segment defined by a label that satisfies this regular expression `^[A-Za-z][0-9A-Za-z\t _-]*:$` and by the presence of equal indentation `^([ \t]+)` on the subsequent lines.

  By default segment is not defined.

- The end of a segment is determined by indentation changes or the end of the file.
- If no segment is defined for a line, the line will be added to each segment defined below.
