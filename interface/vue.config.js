const path = require('path')
function resolve (dir) {
  return path.join(__dirname, dir)
}

module.exports = {
  configureWebpack: {
    devtool: 'nosources-source-map'
  },
  chainWebpack: config => {
    config.resolve.alias
        .set('@', resolve('src/renderer'))
        .set('~', resolve('src'))
        .set('root', resolve('./'))
  },
  pluginOptions: {
    electronBuilder: {
      nodeIntegration: true,
      customFileProtocol: './',
      builderOptions: {
        productName: 'JumpServerClient',
        appId: 'com.jumpserver.client',
        afterSign: "build/sign/notarize.js",
        asar: false,
        extraResources: [
          "bin/**"
        ],
        dmg: {
          sign: false,
          contents: [
            {
              x: 410,
              y: 150,
              type: 'link',
              path: '/Applications'
            },
            {
              x: 130,
              y: 150,
              type: 'file'
            }
          ]
        },
        deb: {
          afterInstall:"build/linux/after-install.sh",
        },
        mac: {
          icon: 'build/icons/icon.icns',
          extendInfo: {
            LSUIElement: 0
          },
          target: [{
            target: 'dmg',
            arch: [
              'x64',
              'arm64'
            ]
          }],
          // eslint-disable-next-line no-template-curly-in-string
          artifactName: 'JumpServer-Client-Installer-${os}-v${version}-${arch}.${ext}',
          protocols: {
            name: "Jms",
            schemes: ["jms"]
          },
        },
        win: {
          icon: 'build/icons/icon.ico',
          // eslint-disable-next-line no-template-curly-in-string
          artifactName: 'JumpServer-Client-Installer-${os}-v${version}-${arch}.${ext}',
          target: [{
            target: 'nsis',
            arch: [
              'x64'
            ]
          }]
        },
        nsis: {
          oneClick: true,
          allowToChangeInstallationDirectory: false,
          deleteAppDataOnUninstall: true,
          include: "build/win/installer.nsh"
        },
        linux: {
          icon: 'build/icons/',
          // eslint-disable-next-line no-template-curly-in-string
          artifactName: 'JumpServer-Client-Installer-${os}-v${version}-${arch}.${ext}',
          target: [{
            target: 'deb',
            arch: [
              'x64',
              'arm64'
            ]
          }]
        },
      },
      chainWebpackMainProcess: (config) => {
        config.resolve.alias
            .set('@', resolve('src/renderer'))
            .set('~', resolve('src'))
            .set('root', resolve('./'))
        config.output.filename((file) => {
          if (file.chunk.name === 'index') {
            return 'background.js';
          } else {
            return '[name].js';
          }
        });
      }
    }
  }
}

