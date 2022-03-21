#! /usr/bin/env sh

cd "$(dirname $0)/.." || exit

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zinstrument-coverage -Zprofile -Ccodegen-units=1 -Copt-level=0 -Coverflow-checks=off"
export LLVM_PROFILE_FILE="default.profraw"

echo 'Run tests'
cargo +nightly test --lib
if [ $? -ne 0 ]; then
  exit 1
fi

mkdir target/coverage >/dev/null 2>&1

echo 'Select files and store them into a Zip archive'
zip -0 ./target/coverage/cov-binaries.zip $(find target -name "poreader*.gc*" -print)

echo 'Produce file for Coveralls'
grcov ./target/coverage/cov-binaries.zip \
  -s . \
  -b target \
  -t coveralls+ \
  --token $COVERALLS_REPO_TOKEN \
  --excl-line 'unreachable' \
  --excl-start '// no-coverage:start' \
  --excl-stop '// no-coverage:stop' \
  --ignore '*/.cargo/*' \
  --ignore '*/target/debug/build/*' \
  --llvm \
  --ignore-not-existing \
  -o target/coverage/coveralls.json
