# mk

Simple text preprocessor for content segmentation.

## Installation

```shell
cargo install --git https://github.com/x0k/mk.git
```

## Syntax

```bash
common content

foo:
    foo segment content

# segment with a dependency
bar: foo
    bar segment content
```

`$ mk bar` output:

```bash
common content

foo segment content

# segment with a dependency
bar segment content
```

## Configuration

- File names must begin with `Mkfile` or `mkfile`.
- Files are read in lexicographic order.
- The contents of the files are concatenated.

Some features configures via file suffixes until first `.` character or end of file name.

- With `x` suffix the result of preprocessing will be saved as tmp file and executed.

## Syntax sugar

### Groups

- `/` at the end of the segment name defines a group.
- `/` at the beginning of the dependency name indicates that name should be left as is.

```makefile

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

```makefile
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

- [glob implementation](https://docs.rs/glob/latest/glob/struct.Pattern.html).

```makefile
l/lib1/build:
    build lib1

l/lib2/build:
    build lib2

app: l/*/build
    build app
```

Desugared:

```makefile
l/lib1/build:
    build lib1

l/lib2/build:
    build lib2

app: l/lib1/build l/lib2/build
    build app
```
