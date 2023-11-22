cargo leptos build --release
mkdir -p bob_release
cp target/release/backend bob_release
cp -r target/site bob_release
cp .env bob_release
cd bob_release
./backend