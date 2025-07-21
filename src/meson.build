# SPDX-FileCopyrightText: 2022  Emmanuele Bassi
# SPDX-License-Identifier: GPL-3.0-or-later

pkgdatadir = get_option('prefix') / get_option('datadir') / meson.project_name()

gnome.compile_resources('amberol',
  'amberol.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

conf = configuration_data()
conf.set_quoted('VERSION', '@0@@1@'.format(meson.project_version(), version_suffix))
conf.set_quoted('GETTEXT_PACKAGE', 'amberol')
conf.set_quoted('LOCALEDIR', get_option('prefix') / get_option('localedir'))
conf.set_quoted('PKGDATADIR', pkgdatadir)
conf.set_quoted('APPLICATION_ID', application_id)
conf.set_quoted('PROFILE', get_option('profile'))

config_rs = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: conf
)

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

# Platform-specific executable name
exe_suffix = host_machine.system() == 'windows' ? '.exe' : ''
amberol_exe = meson.project_name() + exe_suffix

# Set up environment for cargo
cargo_env = environment()
cargo_env.set('CARGO_HOME', meson.project_build_root() / 'cargo-home')

# Build with cargo
cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: 'cargo.stamp',
  console: true,
  env: cargo_env,
  command: [
    cargo, 'build', cargo_options
  ],
)

# Install the built binary
built_exe = meson.project_build_root() / 'src' / rust_target / amberol_exe

install_data(
  built_exe,
  install_dir: get_option('bindir'),
  install_mode: 'rwxr-xr-x',
  depends: cargo_build,
  rename: amberol_exe,
)
