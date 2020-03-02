# https://wiki.archlinux.org/index.php/Rust_package_guidelines

pkgname=paperdump
pkgver=0.1.0
pkgrel=1
pkgdesc="Deal with paperwork quickly so you can move on to something more interesting"
arch=(x86_64)
url="https://github.com/kamek-pf/paperdump"
license=("MIT")
makedepends=("cargo" "clang" "tesseract" "leptonica" "tesseract-data-en")
provides=(${pkgname})
conflicts=(${pkgname})
source=("${pkgname}-${pkgver}.tar.gz::${url}/archive/${pkgver}.tar.gz")
md5sums=(SKIP)

build() {
    cd "${pkgname}-${pkgver}"
    cargo build --release
    strip target/release/${pkgname}
}

package() {
    install -Dm755 "${pkgname}-${pkgver}/target/release/${pkgname}" "${pkgdir}/usr/bin/${pkgname}"
    install -Dm755 "${pkgname}-${pkgver}/config.example.toml" "${pkgdir}/etc/paperdump/config.toml"
}
