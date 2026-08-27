#!/usr/bin/env sh
set -eu

script_dir="$(CDPATH= cd "$(dirname "$0")" && pwd)"
installer="$script_dir/tomat-installer.sh"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/tomat-installer-test.XXXXXX")"
mock_bin="$test_root/bin"
mkdir -p "$mock_bin"
trap 'rm -rf "$test_root"' EXIT INT TERM

fail() {
  echo "installer test failed: $*" >&2
  exit 1
}

cat >"$mock_bin/uname" <<'EOF'
#!/usr/bin/env sh
case "$1" in
-s) printf '%s\n' "$MOCK_UNAME_OS" ;;
-m) printf '%s\n' "$MOCK_UNAME_ARCH" ;;
*) exit 1 ;;
esac
EOF

cat >"$mock_bin/curl" <<'EOF'
#!/usr/bin/env sh
output=""
url=""
while [ "$#" -gt 0 ]; do
  case "$1" in
  -o)
    output="$2"
    shift 2
    ;;
  -*) shift ;;
  *)
    url="$1"
    shift
    ;;
  esac
done

printf '%s\n' "$url" >>"$MOCK_CURL_LOG"
case "$url" in
https://api.github.com/*)
  printf '{"browser_download_url":"https://github.com/jolars/tomat/releases/download/v9.9.9/%s"}\n' "$MOCK_ASSET"
  ;;
*.tar.gz)
  : >"$output"
  ;;
*) exit 22 ;;
esac
EOF

cat >"$mock_bin/tar" <<'EOF'
#!/usr/bin/env sh
destination=""
while [ "$#" -gt 0 ]; do
  case "$1" in
  -C)
    destination="$2"
    shift 2
    ;;
  *) shift ;;
  esac
done

printf '#!/usr/bin/env sh\n' >"$destination/tomat"
chmod +x "$destination/tomat"
EOF

chmod +x "$mock_bin/uname" "$mock_bin/curl" "$mock_bin/tar"

run_case() {
  name="$1"
  os="$2"
  arch="$3"
  libc="$4"
  expected_asset="$5"
  case_dir="$test_root/$name"
  install_dir="$case_dir/install"
  curl_log="$case_dir/curl.log"
  mkdir -p "$case_dir"

  MOCK_UNAME_OS="$os" \
    MOCK_UNAME_ARCH="$arch" \
    MOCK_ASSET="$expected_asset" \
    MOCK_CURL_LOG="$curl_log" \
    TOMAT_LIBC="$libc" \
    TOMAT_INSTALL_DIR="$install_dir" \
    TOMAT_VERIFY_CHECKSUM=false \
    PATH="$mock_bin:$PATH" \
    sh "$installer" >"$case_dir/stdout" 2>"$case_dir/stderr"

  [ -x "$install_dir/tomat" ] || fail "$name did not install an executable"
  expected_url="https://github.com/jolars/tomat/releases/download/v9.9.9/$expected_asset"
  grep -Fx "$expected_url" "$curl_log" >/dev/null \
    || fail "$name selected the wrong release asset"
}

run_case linux-gnu Linux x86_64 gnu tomat-x86_64-unknown-linux-gnu.tar.gz
run_case linux-musl Linux arm64 musl tomat-aarch64-unknown-linux-musl.tar.gz
run_case macos-arm Darwin arm64 "" tomat-aarch64-apple-darwin.tar.gz
run_case macos-intel Darwin amd64 "" tomat-x86_64-apple-darwin.tar.gz

invalid_dir="$test_root/invalid-libc"
mkdir -p "$invalid_dir"
if MOCK_UNAME_OS=Linux \
  MOCK_UNAME_ARCH=x86_64 \
  MOCK_CURL_LOG="$invalid_dir/curl.log" \
  TOMAT_LIBC=other \
  TOMAT_INSTALL_DIR="$invalid_dir/install" \
  PATH="$mock_bin:$PATH" \
  sh "$installer" >"$invalid_dir/stdout" 2>"$invalid_dir/stderr"; then
  fail "an invalid TOMAT_LIBC value succeeded"
fi
grep -F "TOMAT_LIBC must be 'gnu' or 'musl'" "$invalid_dir/stderr" >/dev/null \
  || fail "an invalid TOMAT_LIBC value produced the wrong error"

echo "Installer tests passed."
