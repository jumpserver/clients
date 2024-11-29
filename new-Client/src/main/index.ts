import path, { join, resolve } from 'path';
import { Conf, useConf } from 'electron-conf/main';
import icon from '../../resources/JumpServer.ico?asset';
import { electronApp, is, optimizer } from '@electron-toolkit/utils';
import { app, BrowserWindow, ipcMain, session, shell } from 'electron';

import { execFile } from 'child_process';
import { existsSync, readFileSync } from 'fs';

let mainWindow: BrowserWindow | null = null;

process.env['ELECTRON_DISABLE_SECURITY_WARNINGS'] = 'true';

const configFilePath = path.join(app.getPath('userData'), 'config.json');
let defaults = null;
if (!existsSync(configFilePath)) {
  let subPath = path.join(process.resourcesPath, 'bin');
  if (is.dev) {
    subPath = 'bin';
  }
  const data = readFileSync(path.join(subPath, 'config.json'), 'utf8');
  defaults = JSON.parse(data);
}
const conf = new Conf({ defaults: defaults });

const setDefaultProtocol = () => {
  if (process.defaultApp) {
    if (process.argv.length >= 2) {
      app.setAsDefaultProtocolClient('jms', process.execPath, [resolve(process.argv[1])]);
    }
  } else {
    app.setAsDefaultProtocolClient('jms');
  }
};

const handleUrl = (url: string) => {
  const match = url.match(/^jms:\/\/(.+)$/);
  const token = match ? match[1] : null;

  if (token) {
    const decodedTokenJson = Buffer.from(token, 'base64').toString('utf-8');

    try {
      const decodedToken = JSON.parse(decodedTokenJson);
      mainWindow?.webContents.send('set-token', decodedToken.bearer_token);
    } catch (error) {
      console.error('Failed to parse decoded token:', error);
    }
  }
};

// Window
const handleArgv = (argv: any) => {
  const offset = app.isPackaged ? 1 : 2;
  const url = argv.find((arg, i) => i >= offset && arg.startsWith('jms'));
  if (url) handleUrl(url);
};

const createWindow = async (): Promise<void> => {
  mainWindow = new BrowserWindow({
    width: 1300,
    height: 800,
    show: false,
    frame: false, // 无边框窗口
    center: true,
    autoHideMenuBar: true,
    title: 'JumpServer Client',
    titleBarStyle: 'hidden',
    titleBarOverlay: {
      height: 30,
      color: '#1f1f1f',
      symbolColor: '#ffffff'
    },
    ...(process.platform === 'linux' ? { icon } : { icon }),
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: false,
      webSecurity: false
    }
  });

  mainWindow.on('ready-to-show', () => {
    mainWindow!.show();
  });

  mainWindow.webContents.setWindowOpenHandler(details => {
    shell.openExternal(details.url);
    return { action: 'deny' };
  });

  session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
    const headers = details.responseHeaders;

    // 移除 'Cross-Origin-Opener-Policy' 头
    delete headers?.['Cross-Origin-Opener-Policy'];

    callback({
      cancel: false,
      responseHeaders: headers
    });
  });

  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.webContents.openDevTools();

    const token =
      'eyJ0eXBlIjogImF1dGgiLCAiYmVhcmVyX3Rva2VuIjogImZpRlA3Uk82dzd0ZzhSeWtzRE5qS1NqYVdacjkwMFU4ZFZ4VSIsICJkYXRlX2V4cGlyZWQiOiAxNzMxNjIyMzIwLjgzNjc4Nn0=';

    const decodedTokenJson = Buffer.from(token, 'base64').toString('utf-8');
    const decodedToken = JSON.parse(decodedTokenJson);
    mainWindow?.webContents.send('set-token', decodedToken.bearer_token);

    await mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL']);
  } else {
    await mainWindow.loadFile(join(__dirname, '../renderer/index.html'));
  }
};

!app.requestSingleInstanceLock() ? app.quit() : '';

app.whenReady().then(async () => {
  // Set app user model id for windows
  electronApp.setAppUserModelId('com.jumpserver');

  // Default open or close DevTools by F12 in development
  // and ignore CommandOrControl + R in production.
  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  conf.registerRendererListener();
  useConf();

  await createWindow();
  setDefaultProtocol();

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });

  if (is.dev) {
    if (process.platform === 'win32') {
      process.on('message', data => {
        if (data === 'graceful-exit') {
          app.quit();
        }
      });
    } else {
      process.on('SIGTERM', () => {
        app.quit();
      });
    }
  }
});

// 除外 macOS 外，关闭所有窗口时退出 app
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// @ts-ignore
app.on('second-instance', (_event: Event, argv: string) => {
  if (process.platform === 'win32') {
    handleArgv(argv);
  }
  if (mainWindow) {
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.focus();
  }
});

// 从 web 唤起 Client
// @ts-ignore
app.on('open-url', (_event: Event, url: string) => {
  handleUrl(url);
});

ipcMain.on('open-client', (_, url) => {
  if (url) {
    let subPath = process.resourcesPath + '/bin';
    if (is.dev && !process.env.IS_TEST) {
      subPath = 'bin';
    }
    if (process.platform === 'linux') {
      switch (process.arch) {
        case 'x64':
          subPath += '/linux-amd64';
          break;
        case 'arm':
        case 'arm64':
          subPath += '/linux-arm64';
          break;
      }
    } else if (process.platform === 'darwin') {
      subPath += '/darwin';
    } else {
      subPath += '/windows';
    }
    let exeFilePath = path.join(subPath, 'JumpServerClient');
    execFile(exeFilePath, [url], error => {
      if (error) {
        console.log(error);
      }
    });
  }
});

const callBackUrlName = 'config-reply-get'; //消息发布-发布名称
//读取本地文件
ipcMain.on('config-get', function (event) {
  // 传给渲染进程数据
  fse.readFile(configFilePath, 'utf8', (err, data) => {
    if (err) {
      event.sender.send(callBackUrlName, 500, '读取 config.json 文件失败');
    } else {
      event.sender.send(callBackUrlName, 200, data);
    }
  });
});

//增改本地文件
ipcMain.on('config-set', function (event, type, value) {
  value = JSON.parse(value);
  fse.readFile(configFilePath, 'utf8', (err, data) => {
    if (err) {
      console.log('目标文件异常');
    } else {
      let config = JSON.parse(data);
      let platform;
      if (process.platform === 'win32') {
        platform = 'windows';
      } else if (process.platform === 'darwin') {
        platform = 'macos';
      } else {
        platform = 'linux';
      }
      let lst = [];
      switch (type) {
        case 'sshPage':
          lst = config[platform]['terminal'];
          break;
        case 'remotePage':
          lst = config[platform]['remotedesktop'];
          break;
        case 'fileTransferPage':
          lst = config[platform]['filetransfer'];
          break;
        case 'databasesPage':
          lst = config[platform]['databases'];
          break;
      }
      lst.forEach(item => {
        if (value.is_default) {
          item.is_default = false;
        }
        if (item.match_first.length > 0) {
          item.match_first = item.match_first.filter(item => !value.match_first.includes(item));
        }
        if (item.name === value.name) {
          item.path = value.path;
          item.is_default = value.is_default;
          item.is_set = value.is_set;
          item.match_first = value.match_first;
        }
      });
      const config_str = JSON.stringify(config);
      fse.writeFileSync(configFilePath, config_str, 'utf8');
      event.sender.send(callBackUrlName, 200, config_str);
    }
  });
});
