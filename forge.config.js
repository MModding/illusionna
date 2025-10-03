module.exports = {
  packagerConfig: {
    asar: true,
    protocols: [
        {
            name: "Illusionna",
            schemes: ["illusionna"]
        }
    ]
  },
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {},
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
    {
      name: '@electron-forge/maker-deb',
      config: {
          mimeType: ["x-scheme-handler/illusionna"]
      },
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {},
    },
  ],
  plugins: [
    {
      name: '@electron-forge/plugin-vite',
      config: {
        // `build` can specify multiple entry builds, which can be Main process, Preload scripts, Worker process, etc.
        // If you are familiar with Vite configuration, it will look really familiar.
        build: [
          {
            // `entry` is just an alias for `build.lib.entry` in the corresponding file of `config`.
            entry: 'src-electron/main.js',
            config: 'src-config/vite.main.config.mjs',
            target: 'main',
          },
          {
            entry: 'src-electron/preload.js',
            config: 'src-config/vite.preload.config.mjs',
            target: 'preload',
          },
        ],
        renderer: [
          {
            name: 'main_window',
            config: 'src-config/vite.renderer.config.mjs',
          },
        ],
      },
    }
  ],
};
