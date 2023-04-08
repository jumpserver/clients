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
        .set('#', resolve('src/universal'))
  },
  pluginOptions: {
    electronBuilder: {
      removeElectronJunk: false,
      builderOptions: {
        productName: 'JumpServer Clients',
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
            LSUIElement: 1
          },
          target: [{
            target: 'dmg',
            arch: [
              'x64',
              'arm64'
            ]
          }],
          // eslint-disable-next-line no-template-curly-in-string
          artifactName: 'JumpServer-Clients-${version}-${arch}.dmg'
        },
        win: {
          icon: 'build/icons/icon.ico',
          // eslint-disable-next-line no-template-curly-in-string
          artifactName: 'JumpServer-Clients-${version}-${arch}.exe',
          target: [{
            target: 'msi',
            arch: [
              'x64',
              'ia32'
            ]
          }]
        },
        linux: {
          icon: 'build/icons/'
        },
        snap: {
          publish: ['github']
        }
      }
    }
  }
}

