export LEPTOS_SITE_ROOT=target/site-release

cargo leptos build --release
./target/release/backend 