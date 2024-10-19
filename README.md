# mk

Simple text preprocessor for content segmentation.

## Installation

### With `cargo`

```shell
cargo install --git https://github.com/x0k/mk.git
```

### With `nix profile`

```shell
nix profile install github:x0k/mk
```

## Syntax

`mkfile` content:

```bash
common content

foo:
    foo segment start

# segment with a dependency
bar: foo
    bar segment content

# multiple segment definition
foo:
    foo segment end
```

`$ mk bar | cat` output:

```bash
common content

foo segment start

# segment with a dependency
bar segment content

# multiple segment definition
foo segment end
```

That's all.

## Input

- Content can be specified via std in or input files.
- By default, input file names should begin with `Mkfile` or `mkfile`.
- Files are read in lexicographic order.
- The contents of the files are concatenated.

## Output

- By default, a temporary file will be created and executed.
- If you pipe the program to something then the std out will be used.

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

## Completions

> [!NOTE]
> Completions ignore the `input` argument.

Source completions with `bash`:

`echo "source <(COMPLETE=bash mk)" >> ~/.bashrc`

For other shells see: [clap_complete](https://docs.rs/clap_complete/4.5.33/clap_complete/env/index.html)

## License

MIT
