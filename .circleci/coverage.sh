#! /usr/bin/env sh

cd "$(dirname $0)/.." || exit

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zinstrument-coverage -Zprofile -Ccodegen-units=1 -Copt-level=0 -Coverflow-checks=off"
export LLVM_PROFILE_FILE="default.profraw"

echo 'Run tests'
if cargo +nightly test; then
  mkdir target/coverage >/dev/null 2>&1

  echo 'Select files and store them into a Zip archive'
  zip -0 ./target/coverage/ccov.zip `find target \( -name "poreader*.gc*" \) -print`
  cp ./target/coverage/ccov.zip /tmp/artifacts/

  echo 'Produce file for Coveralls'
  grcov ./target/coverage/ccov.zip \
    -s src \
    -t lcov \
    --ignore '*/.cargo/*' \
    --ignore '*/target/debug/build/*' \
    --llvm \
    --ignore-not-existing \
    | awk '/^SF:/ {printf "SF:/home/circleci/project/%s", substr($0, 4); print ""; next} {print}' \
    > target/coverage/lcov.info
fi
