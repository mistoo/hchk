Summary:	healthchecks.io command line client
Name:		hchk
Version:	0.1.1
Release:	1
License:	MIT
Group:		Applications/System
# URL https://github.com/mistoo/%{name}/v%{version}.tar.gz
Source0:	v%{version}.tar.gz
URL:		https://github.com/mistoo/%{name}
BuildRequires:	cargo
BuildRequires:	rust
BuildRoot:	%{tmpdir}/%{name}-%{version}-root-%(id -u -n)

%define		_enable_debug_packages 0

%description
healthchecks.io command line client

%prep
%setup -q

%build
cargo build --release

%install
rm -rf $RPM_BUILD_ROOT
install -d $RPM_BUILD_ROOT%{_bindir}
install -p target/release/%{name} $RPM_BUILD_ROOT%{_bindir}

%clean
rm -rf $RPM_BUILD_ROOT

%files
%defattr(644,root,root,755)
%doc README.md
%attr(755,root,root) %{_bindir}/%{name}
