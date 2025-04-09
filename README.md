# PingPong 🏓

PingPong allows you to perform API requests directly from terminal through an UI (TermUI). Think of Postman but in the terminal, but PingPong also allows you to run load tests to an endpoint. Gradually increase requests per second until server starts returning > 20% error rate.

## Setup

You can run PingPong through [Cargo command](https://rustup.rs/) and then simply run the following command:

```sh
cargo run
```

This should spin up the UI

## Demo

There is a sample bun server included that you can use to test the UI. Checkout the recording below.

<video width="320" height="240" controls>
  <source src="https://github.com/grohith327/PingPong/blob/main/assets/ping_pong_demo.mov" type="video/mp4">
</video>
