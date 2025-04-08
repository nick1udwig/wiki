# sign

A fork of [hyperdrive's `sign`](https://github.com/hyperware-ai/hyperdrive/tree/4d5223758087d2813f3598c69907306e953dbab1/hyperdrive/packages/sign) using the Hyperapp Framework.

## Goals

1. Get Hyperapp Framework into a more full-featured, robust state by serving as a testing ground for it & its `kit` integration,
2. Begin the move of core apps to Hyperapp Framework.

## Usage

Use [`hf/build-add-hyper-bindgen` branch of `kit`](https://github.com/hyperware-ai/kit/pull/312) i.e.
```
cargo install --git https://github.com/hyperware-ai/kit --locked --branch hf/build-add-hyper-bindgen
```

Build using
```
kit b --hyperapp
```

## Current state & TODOs

1. Compilation is working as of [kit@4ad1b14](https://github.com/hyperware-ai/kit/pull/312/commits/4ad1b14bd730c210040757e8eed4e25d70ba6955),
2. No promises as to functionality yet,
3. Next step is to add `caller-utils` generation into the dependency system and write a minimal app that calls `sign:sign:sys`,
4. Subsequent step is to probably rip out the `SendResult` and make it into an actual `Result`.
