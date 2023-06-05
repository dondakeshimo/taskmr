# taskmr
task manager for CLI

# Install

```
cargo install taskmr
```

# Usage

```
$ taskmr help
```

# What is `es-` prefix command?

taskmr has been devoloped by two design patterns for educational purposes:

1. A simple table structure
1. Event Sourcing

Files prefixed with `es_` contain code implemented according to the Event Sourcing pattern.

Please note that these two implementations use separate databases, hence, it's not possible to manage the same task in both `es-` and non-`es-` versions.

# How to develop

See [how-to-develop.md](./docs/how-to-develop.md)
