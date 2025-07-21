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

# Build profile
if get_option('profile') == 'default'
  build_profile = 'release'
  message('Building in release mode')
else
  build_profile = 'debug'
  message('Building in debug mode')
endif

# Platform-specific executable name
exe_suffix = host_machine.system() == 'windows' ? '.exe' : ''
output_name = meson.project_name() + exe_suffix

# Find Python
python = find_program('python3', 'python')

cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: output_name,
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    python, meson.project_source_root() / 'build_cargo.py',
    '--cargo-home', meson.project_build_root() / 'cargo-home',
    '--manifest-path', meson.project_source_root() / 'Cargo.toml',
    '--target-dir', meson.project_build_root() / 'src',
    '--profile', build_profile,
    '--output', '@OUTPUT@',
    '--project-name', meson.project_name(),
  ],
)
