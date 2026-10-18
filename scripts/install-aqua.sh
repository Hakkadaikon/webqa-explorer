#!/bin/sh
# Install the pinned aqua binary into .aqua/bin, verifying its sha256.
# Every other developer tool is then installed by `aqua install` (aqua.yaml).
set -eu

version=2.64.0
root=$(cd "$(dirname "$0")/.." && pwd)
dest="$root/.aqua/bin"

case "$(uname -s)/$(uname -m)" in
Linux/x86_64) asset=linux_amd64 sum=b276e927ee2f4dc214831708c0843a91007bafa06f2f9fcd6ee1fd73fd165598 ;;
Linux/aarch64) asset=linux_arm64 sum=7cc2b859333f40ef52eb62a1be3bfaa34dcbc27ef1c8bdc5bfaea7787597bc18 ;;
Darwin/x86_64) asset=darwin_amd64 sum=64bc676871f2fe2683f57371e958963fc5a14b72b03e22b796fa228dcdeb353b ;;
Darwin/arm64) asset=darwin_arm64 sum=f7f7890781bbd8863b04b61eb7bb40fe83add7097243c13da987ae9fb56d93b7 ;;
*)
	echo "install-aqua: unsupported platform $(uname -s)/$(uname -m)" >&2
	exit 1
	;;
esac

if [ -x "$dest/aqua" ] && "$dest/aqua" version | grep -q "aqua version $version"; then
	exit 0
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
curl -sSfL -o "$tmp/aqua.tar.gz" \
	"https://github.com/aquaproj/aqua/releases/download/v$version/aqua_$asset.tar.gz"
echo "$sum  $tmp/aqua.tar.gz" | sha256sum -c - >/dev/null
tar -xzf "$tmp/aqua.tar.gz" -C "$tmp" aqua
mkdir -p "$dest"
mv "$tmp/aqua" "$dest/aqua"
echo "installed aqua $version to $dest"
