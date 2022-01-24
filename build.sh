source $stdenv/setup

tar xvfz $src
cd GoodBoy-*

cargo install --root $out --path .
