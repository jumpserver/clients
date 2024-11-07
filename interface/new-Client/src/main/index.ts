import { join, resolve } from 'path';
import { Conf, useConf } from 'electron-conf/main';
import icon from '../../resources/JumpServer.ico?asset';
import { electronApp, optimizer, is } from '@electron-toolkit/utils';
import { app, shell, BrowserWindow, ipcMain, session } from 'electron';

let mainWindow: BrowserWindow | null = null;

process.env['ELECTRON_DISABLE_SECURITY_WARNINGS'] = 'true';

const conf = new Conf();

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

  // IPC test
  ipcMain.on('ping', () => console.log('pong'));

  conf.registerRendererListener();
  useConf();

  // 设置一些默认全局配置
  conf.set('defaultSetting', {
    // 默认的布局
    layout: 'list',
    // 默认的语言
    language: 'en',
    // 默认的主题
    theme: 'light'
  });

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
