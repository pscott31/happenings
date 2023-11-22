cargo leptos build 
mkdir -p bob
cp target/debug/backend bob
cp -r target/site bob
cp .env bob
cd bob
./backend
