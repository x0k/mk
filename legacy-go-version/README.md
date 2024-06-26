# mk

Text preprocessor for content segmentation.

## Installation

### Via `go install`

With go 1.18 or higher:

```shell
go install github.com/x0k/mk@latest
```

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

### X suffix

If file contains `x` suffix, then result of preprocessing will be saved as tmp file and executed.

`mkfilex` file

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

### Targets

Each segment can define its targets.

If the segment has a target that is a prefix of current target segment, it will be included in the result.

`mkfilex` file

```shell
#!/usr/bin/bash
prepare: test build
  echo "prepare"
build:
  echo "build"
test:
  echo "test"
clean:
  echo "clean"
```

`mk build` output

```shell
prepare
build
```

`mk clean` output

```shell
clean
```

The equivalent `mkfilex`

```shell
#!/usr/bin/bash
clean:
  echo "clean"
echo "prepare"
build:
  echo "build"
test:
  echo "test"
```

### Default target

By default target segment is `default` and all segments are implicitly declare `default` as their target.
So `$ mk` command will include all segments in the result.

If you want to exclude some segment from `$ mk` result, then:
- define segment with target (any non-empty string/strings not beginning with `default`)
- define segment after `default` segment.

`mkfilex` file

```shell
#!/usr/bin/bash
build:
  echo "build"
test: build
  echo "test"
default:
clean:
  echo "clean"
```

`mk` output

```shell
build
```

## Details

- The segment defined by a label (and optional targets) that satisfies this regular expression `^([A-z][0-9A-z\._-]*):(.*)$` and by the presence of equal indentation `^([ \t]+)` on the subsequent lines.

  By default segment is not defined.

- Segment targets are a whitespace-separated list of segment prefixes that follow the `:` character.
- The end of a segment is determined by indentation changes or the end of the file.
- If segment is not defined for a line, the line will be added to each segment defined below.
- The filename must match the regular expression `^(M|m)kfile.*$`, all matching files are processed in reverse lexicographic order until the target segment is found.
