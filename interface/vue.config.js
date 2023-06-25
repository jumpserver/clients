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
      nodeIntegration:true,
      customFileProtocol: './',
      builderOptions: {
        productName: 'JumpServer Config Clients',
        appId: 'com.jumpserver.client',
        dmg: {
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
          artifactName: 'JumpServer-Config-Clients-${version}-${arch}.dmg'
        },
        win: {
          icon: 'build/icons/icon.ico',
          // eslint-disable-next-line no-template-curly-in-string
          artifactName: 'JumpServer-Config-Clients-${version}-${arch}.exe',
          target: [{
            target: 'nsis',
            arch: [
              'x64',
              'ia32'
            ]
          }]
        },
        nsis: {
          oneClick: false,
          allowToChangeInstallationDirectory: true,
        },
        linux: {
          icon: 'build/icons/'
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

