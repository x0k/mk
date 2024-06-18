# mk

Simple text preprocessor for content segmentation.

## Basic syntax

```bash
common content

foo:
    foo segment content

# segment with a dependency
bar: foo
    bar segment content
```

`$ mk bar` command output:

```bash
common content

foo segment content

# segment with a dependency
bar segment content
```

## Configuration

Configuration is done via file suffixes.

- With `x` suffix the result of preprocessing will be saved as tmp file and executed.

## Syntax sugar

### Groups

```bash

foo:
    foo content

group/:
    pushd folder

    bar: /foo
        bar segment content

    baz: bar
        baz segment content

    popd
```

Desugared:

```bash
foo:
    foo content

group:
    pushd folder

group/bar: foo group
    bar segment content

group/baz: group/bar group
    baz segment content

group:
    popd
```

### Glob pattern in dependencies list

```bash
l/lib1/build:
    build lib1

l/lib2/build:
    build lib2

app: l/*/build
    build app
```

Desugared:

```bash
l/lib1/build:
    build lib1

l/lib2/build:
    build lib2

app: l/lib1/build l/lib2/build
    build app
```
