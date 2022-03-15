# How to develop
This document describe the way to build, to test and to run codes.
Is also mention some rule about development of taskmr.


## Development Flow
`main` branch is the trunk. PR will be always based on `main`.
To merge PR, you must get 1 approval and pass GitHub Actions check.


## Modeling
It is very simple to describe this project.

### Use cases
<img width="953" alt="スクリーンショット 2022-03-16 0 19 11" src="https://user-images.githubusercontent.com/23194960/158411002-4373fc37-f62f-46bb-b1bd-ffc581c23a48.png">

### Domain model
<img width="645" alt="スクリーンショット 2022-03-16 0 19 18" src="https://user-images.githubusercontent.com/23194960/158411115-783b5086-e17a-466e-b1a0-1c6dfb03d7c1.png">


## Architecture
The module structure is based on Hexagonal Architecture and Onion Architecture.
Domain events drive procceses. The pattern may be overdoing, but I want to use.

Data is stored by SQLite because it is the best choice to store data locally.


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
