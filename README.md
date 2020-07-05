# Ecommerce API, Real World Example with Rust (actix-web as framework and mongodb as database)

I think that the examples and resources of developing async web services in Rust are scarce. So I decided to open-source an API which
I developed for an e-commerce site using [actix-web framework](https://github.com/actix/actix-web) a couple of months ago. I hope it gives an idea to those who wants to develop async web services using Rust Programming Language.

This API is literally real-wold example, it is live [here](https://www.koydenevine.com)

## Features
- `Create, Read, Update` users
- JWT Authentication middleware
- `Read` listings
- `Create, Read, Update` addresses
- `Create, Read, Update, Delete` basket
- `Create, Read` orders

## How to run

You can either create a docker image, or run it locally.

- create a prod.env file, copy content of .env file to prod.env file, and change them as you wish

#### to create a docker container
- `docker build -t you-name-it .`
- `docker run --env-file ./prod.env --publish 3003:3003 --name name-your-container you-name-it`

container named `name-your-container` will start.

#### to run locally

- `cargo run --release`

app will start at port 3003.

---

***Stars and PR's are welcome!***