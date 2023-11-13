<picture>
    <source srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_Solid_White.svg" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg" alt="Leptos Logo">
</picture>

# Leptos Axum Starter Template

This is a template for use with the
[Leptos](https://github.com/leptos-rs/leptos) web framework and the
[cargo-leptos](https://github.com/akesson/cargo-leptos) tool using
[Axum](https://github.com/tokio-rs/axum).

## Creating your template repo

If you don't have `cargo-leptos` installed you can install it with

```bash
cargo install cargo-leptos
```

Then run

```bash
cargo leptos new --git leptos-rs/start-axum
```

to generate a new project template.

```bash
cd nexus
```

to go to your newly created project.\
Feel free to explore the project structure, but the best place to start with
your application code is in `src/app.rs`.\
Addtionally, Cargo.toml may need updating as new versions of the dependencies
are released, especially if things are not working after a `cargo update`.

## Running your project

```bash
cargo leptos watch
```

## Installing Additional Tools

By default, `cargo-leptos` uses `nightly` Rust, `cargo-generate`, and `sass`. If
you run into any trouble, you may need to install one or more of these tools.

1. `rustup toolchain install nightly --allow-downgrade` - make sure you have
   Rust nightly
2. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust
   to WebAssembly
3. `cargo install cargo-generate` - install `cargo-generate` binary (should be
   installed automatically in future)
4. `npm install -g sass` - install `dart-sass` (should be optional in future

## Compiling for Release

```bash
cargo leptos build --release
```

Will generate your server binary in target/server/release and your site package
in target/site

## Testing Your Project

```bash
cargo leptos end-to-end
```

```bash
cargo leptos end-to-end --release
```

Cargo-leptos uses Playwright as the end-to-end test tool.\
Tests are located in end2end/tests directory.

## Executing a Server on a Remote Machine Without the Toolchain

After running a `cargo leptos build --release` the minimum files needed are:

1. The server binary located in `target/server/release`
2. The `site` directory and all files within located in `target/site`

Copy these files to your remote server. The directory structure should be:

```text
nexus
site/
```

Set the following environment variables (updating for your project as needed):

```text
LEPTOS_OUTPUT_NAME="nexus"
LEPTOS_SITE_ROOT="site"
LEPTOS_SITE_PKG_DIR="pkg"
LEPTOS_SITE_ADDR="127.0.0.1:3000"
LEPTOS_RELOAD_PORT="3001"
```

Finally, run the server binary.

## Deploying Your Project

To build and deploy your project to AWS, you'll need
[cargo-lambda](https://www.cargo-lambda.info/). They provide
[installation instructions](https://www.cargo-lambda.info/guide/installation.html)
on their site.

Let's start by building the project with `cargo-leptos`:

```bash
cargo leptos build --release
```

We won't use the server binary that it builds, since the Lambda function
requires a particular architecture that `cargo-lambda` will handle for us. If
you'd rather not build the server twice, you'll have to manage the wasm build
and optimization yourself.

Next, let's build the production server binary:

```bash
LEPTOS_OUTPUT_NAME=aws-lambda cargo lambda build --no-default-features --features=ssr --release
```

This should produce a binary at `target/lambda/aws-lambda/bootstrap`.
`Cargo.toml` exposes all the required environment variables to `cargo-lambda` so
that the server can run in production.

Finally, we can deploy the project to AWS:

```bash
cargo lambda deploy --include target/site --enable-function-url
```

After a few seconds, `cargo-lambda` should print out the URL of your deployed
function!

## Notes

### Credentials

You'll need AWS credentials with some permissions for IAM and Lambda operations.
`cargo-lambda` provides the
[minimum requirements here](https://www.cargo-lambda.info/commands/deploy.html#user-profile).

Setting up permissions can be a bit onerous if this is your first time working
with AWS. For a quick and dirty setup, you can:

1. Create a new user in the IAM service (Access Management > Users)
2. Click "Attach policies directly" on the "Set permissions" page
3. Add the "AWSLambda_FullAccess" and "IAMFullAccess" policies, and complete the
   user creation
4. Create an access key for the user (don't worry about the warning)
5. Place the access key and secret key in `~/.aws/credentials` (or wherever the
   appropriate location is for your system):

```
[default]
aws_access_key_id = AKIAQYLPMN5HCTNK35FD
aws_secret_access_key = rbWHpaI/lJnXdLteWHNnTVZpQztMB2+pdbb+KVgr
```

### Optimizations

Serving static files from a lambda function is not the best approach. Ideally,
you should upload your files to a CDN and configure your project to serve them
from that location. AWS has an article on deploying
[React with SSR](https://aws.amazon.com/blogs/compute/building-server-side-rendering-for-react-in-aws-lambda/).

It's also pretty easy to set up edge compute with Lambda@Edge, which should
improve latency.
