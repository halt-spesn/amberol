# SPDX-FileCopyrightText: 2022  Emmanuele Bassi
# SPDX-License-Identifier: GPL-3.0-or-later

project('amberol', 'rust',
  version: '2024.2',
  license: ['GPL-3.0'],
  meson_version: '>= 0.59.0',
  default_options: [ 'warning_level=2', ],
)

# Platform detection
is_windows = host_machine.system() == 'windows'
is_linux = host_machine.system() == 'linux'

# Dependencies
dependency('gtk4', version: '>= 4.13.4')
dependency('libadwaita-1', version: '>= 1.5')
dependency('gstreamer-1.0', version: '>= 1.20')
dependency('gstreamer-audio-1.0', version: '>= 1.20')
dependency('gstreamer-player-1.0', version: '>= 1.20')
dependency('gstreamer-plugins-base-1.0', version: '>= 1.20')

# Platform-specific dependencies
if is_windows
  # Additional Windows dependencies
  message('Building for Windows')
else
  dependency('gstreamer-plugins-bad-1.0', version: '>= 1.20')
  dependency('gstreamer-bad-audio-1.0', version: '>= 1.20')
endif

i18n = import('i18n')
gnome = import('gnome')
fs = import('fs')

cargo = find_program('cargo', required: true)

cargo_sources = files(
  'Cargo.toml',
  'Cargo.lock',
)

if get_option('profile') == 'development'
  profile = '.Devel'
  if fs.is_dir('.git')
    vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: true).stdout().strip()
    if vcs_tag == ''
      version_suffix = '-devel'
    else
      version_suffix = '-@0@'.format(vcs_tag)
    endif
  else
    version_suffix = '-devel'
  endif
else
  profile = ''
  version_suffix = ''
endif

application_id = 'io.bassi.Amberol@0@'.format(profile)

subdir('data')
subdir('src')
subdir('po')

reuse = find_program('reuse', required: false)
if reuse.found()
  test('license',
    reuse,
    args:['lint'],
    workdir: meson.project_source_root(),
    suite: ['lint'],
  )
endif

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_source_root(),
  meson.project_build_root() / 'meson-dist' / '@0@-@1@'.format(meson.project_name(), meson.project_version()),
)

# Platform-specific post-install tasks
if is_windows
  # Skip GNOME-specific post-install tasks on Windows
  message('Skipping GNOME post-install tasks on Windows')
else
  gnome.post_install(
    glib_compile_schemas: true,
    gtk_update_icon_cache: true,
    update_desktop_database: true,
  )
endif

summary({
    'prefix': get_option('prefix'),
    'libdir': get_option('libdir'),
    'datadir': get_option('datadir'),
    'bindir': get_option('bindir'),
  },
  section: 'Directories',
)

summary({
    'Profile': get_option('profile'),
    'Platform': host_machine.system(),
  },
  section: 'Build options',
)
