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

1. Working end-to-end with [id](https://github.com/nick1udwig/id) as of [kit@] [sign@] [id@].
2. Use as follows:
   ```
   # Get proper version of kit
   cargo install --git https://github.com/hyperware-ai/kit --locked --branch hf/build-add-hyper-bindgen

   # Start fake node
   kit f

   # In new terminal
   git clone https://github.com/nick1udwig/sign.git
   cd sign
   kit b --hyperapp
   kit s
   cd ..

   git clone https://github.com/nick1udwig/id.git
   cd id
   kit b --hyperapp -p 8080
   kit s
   cd ..
   ```
   Then open localhost:8080/id:id:sys and set your fake node terminal to Event Loop verbosity (hit Ctrl + V 3 times in the fake node terminal).
