Name: $PACKAGE_NAME
Release: 1%{?dist}
Version: $PACKAGE_VERSION
Summary: $PACKAGE_DESCRIPTION
License: $PACKAGE_LICENSE

%description
A loadable builtin that records the resources used by every executed program,
and report them in a custom format string (like GNU time) or in JSON.

%build
cargo build --release

%install
install -t %{buildroot}/usr/lib/bash -s -D -o root -g root target/release/libtimehistory_bash.so

%files
/usr/lib/bash/libtimehistory_bash.so
