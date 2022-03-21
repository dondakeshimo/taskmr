# How to develop
This document describe the way to build, to test and to run codes.
Is also mention some rule about development of taskmr.


## Development Flow
`main` branch is the trunk. PR will be always based on `main`.
To merge PR, you must get 1 approval and pass GitHub Actions check.


## Modeling
It is very simple to describe this project.

### Use cases
<img width="957" alt="スクリーンショット 2022-03-21 18 15 49" src="https://user-images.githubusercontent.com/23194960/159232847-b15f1209-cd5f-4029-9a60-38f4fc748cd0.png">

### Domain model
<img width="645" alt="スクリーンショット 2022-03-16 0 19 18" src="https://user-images.githubusercontent.com/23194960/158411115-783b5086-e17a-466e-b1a0-1c6dfb03d7c1.png">


## Architecture
The module structure is based on Onion Architecture.

The other keywords about architecture are bellow.

- Domain events
- Event sourcing

These patterns may be overdoing, but **I want to use**.

I use SQLite as database. I think it is the best choice to store data locally.


## Build
It's easy to build with cargo.

```
cargo build
```


## Run
You can use cargo run.

```
cargo run
```

If you want to use target binary, you should build and then run bellow.

```
cargo build

taskmr -h
```

## Test
Cargo includes test command.

```
cargo test
```

If you want to specify the test.

```
cargo test specific_test_name
```

## Style Guide

### Documentation comment
Documentation comments are required. Don't forget to add.

### Unit test
Unit test is required. It is basically supposed to implement within the same file written target codes,
but if it is uncomfortable to live together test and target codes, you can write these separately.

### Argument prefix
I suggest you add a prefix `a_` to arguments if needed. This rule dosen't have to apply all arguments.
Rust allow short-hand syntax with define a struct, then to use it, arguments as the same name as fields are efficient.
