import path from 'path';
import log from 'electron-log';
import icon from '../../resources/JumpServer.ico?asset';

import { execFile } from 'child_process';
import { existsSync, readFileSync } from 'fs';
import { Conf, useConf } from 'electron-conf/main';
import { electronApp, is, optimizer } from '@electron-toolkit/utils';
import { app, BrowserWindow, ipcMain, session, shell } from 'electron';

process.env['ELECTRON_DISABLE_SECURITY_WARNINGS'] = 'true';

let defaults = {
  windowBounds: {
    width: 1280,
    height: 800
  },
  defaultSetting: {
    theme: 'light',
    layout: 'list',
    language: 'en'
  }
};

let mainWindow: BrowserWindow | null = null;

let openMainWindow = false;
let lastSentToken = '';
let lastSentTime = 0;

// prettier-ignore
const platform = process.platform === 'win32' ? 'windows' : process.platform === 'darwin' ? 'macos' : 'linux';
const configFilePath = path.join(app.getPath('userData'), 'config.json');

if (!existsSync(configFilePath)) {
  let subPath = path.join(process.resourcesPath);

  if (is.dev) {
    subPath = 'bin';
  }

  const data = readFileSync(path.join(subPath, 'config.json'), 'utf8');
  defaults = JSON.parse(data);
}

const conf = new Conf({ defaults: defaults! });

const setDefaultProtocol = () => {
  if (process.defaultApp) {
    if (process.argv.length >= 2) {
      app.setAsDefaultProtocolClient('jms', process.execPath, [path.resolve(process.argv[1])]);
    }
  } else {
    app.setAsDefaultProtocolClient('jms');
  }
};

const handleUrl = (url: string) => {
  const match = url.match(/^jms:\/\/(.+)$/);
  const session = match ? match[1] : null;

  if (session) {
    const decodedSessionJson = Buffer.from(session, 'base64').toString('utf-8');

    try {
      const decodedSession = JSON.parse(decodedSessionJson);

      if ('type' in decodedSession) {
        openMainWindow = true;
        mainWindow?.webContents.send('set-login-session', decodedSession.type);
      } else {
        openMainWindow = false;
        handleClientPullUp(url);
      }

      log.info('handleUrl:', openMainWindow);
    } catch (error) {
      log.error('Failed to parse decoded session:', error);
    }
  }
};

const handleArgv = (argv: string[]) => {
  const offset = app.isPackaged ? 1 : 2;
  const url = argv.find((arg, i) => i >= offset && arg.startsWith('jms'));
  if (url) handleUrl(url);
};

const handleClientPullUp = (url: string) => {
  if (url) {
    let subPath = process.resourcesPath;
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
    const exeFilePath = path.join(subPath, 'JumpServerClient');
    execFile(exeFilePath, [url], error => {
      if (error) {
        console.log(error);
      }
    });
  }
};

const createWindow = async (): Promise<void> => {
  const windowBounds =
    (conf.get('windowBounds') as { width: number; height: number }) || defaults.windowBounds;

  mainWindow = new BrowserWindow({
    width: windowBounds.width,
    height: windowBounds.height,
    show: false,
    frame: false, // 无边框窗口
    center: true,
    autoHideMenuBar: true,
    title: 'JumpServer Client',
    titleBarStyle: 'hidden',
    ...(process.platform === 'linux' ? { icon } : { icon }),
    ...(process.platform !== 'darwin' ? { titleBarOverlay: true } : {}),
    webPreferences: {
      preload: path.join(__dirname, '../preload/index.js'),
      sandbox: false,
      webSecurity: false
    }
  });

  mainWindow.on('ready-to-show', () => {
    mainWindow!.show();
  });

  mainWindow.webContents.setWindowOpenHandler(details => {
    try {
      shell.openExternal(details.url);
    } catch (err) {
      log.error('Failed to open external URL:', err);
    }
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

  mainWindow.on('close', () => {
    try {
      if (!mainWindow?.isDestroyed()) {
        const bounds = mainWindow?.getBounds();
        conf.set('windowBounds', bounds);
      }
    } catch (error) {
      console.error('Error saving window bounds:', error);
    }
  });

  mainWindow.on('resize', () => {
    try {
      if (!mainWindow?.isDestroyed()) {
        const bounds = mainWindow?.getBounds();
        conf.set('windowBounds', bounds);
      }
    } catch (error) {
      console.error('Error saving window bounds on resize:', error);
    }
  });

  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.webContents.openDevTools();

    await mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL']);
  } else {
    await mainWindow.loadFile(path.join(__dirname, '../renderer/index.html'));
  }
};

// @ts-ignore
app.on('second-instance', (_event: Event, argv: string[]) => {
  log.info('second-instance');
  if (process.platform === 'win32' || process.platform === 'linux') {
    handleArgv(argv);
  }
  if (mainWindow && openMainWindow) {
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.focus();
  }
});

// @ts-ignore
app.on('open-url', (_event: Event, url: string) => {
  log.info('open-url');
  handleUrl(url);
});

!app.requestSingleInstanceLock() ? app.quit() : '';

app.whenReady().then(async () => {
  // Set app user model id for windows
  electronApp.setAppUserModelId('com.jumpserver');

  // Default open or close DevTools by F12 in development
  // and ignore CommandOrControl + R in production.
  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  app.on('certificate-error', (event, webContents, url, error, certificate, callback) => {
    //允许私有证书
    event.preventDefault();
    callback(true);
  });

  conf.registerRendererListener();

  useConf();

  setDefaultProtocol();

  if (process.platform === 'win32' || process.platform === 'linux') {
    handleArgv(process.argv);
  }

  log.info('whenReady: ', openMainWindow);

  if (openMainWindow) {
    await createWindow();
    setTitleBar(conf.get('defaultSetting.theme') as string);
  } else {
    app.quit();
  }

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0 && openMainWindow) createWindow();
  });

  if (is.dev) {
    if (process.platform === 'win32' || process.platform === 'linux') {
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
  // mainWindow 可能已经被销毁，所以不要再尝试访问它
  app.quit();
});

ipcMain.on('update-titlebar-overlay', (_, theme) => {
  setTitleBar(theme);
});
ipcMain.on('open-client', (_, url) => {
  handleClientPullUp(url);
});
ipcMain.on('get-platform', function (event) {
  event.sender.send('platform-response', platform);
});
ipcMain.on('get-app-version', function (event) {
  event.sender.send('app-version-response', app.getVersion());
});

const setTitleBar = (theme: string) => {
  if (mainWindow && process.platform !== 'darwin') {
    theme === 'dark'
      ? mainWindow.setTitleBarOverlay({
          color: '#00000000',
          symbolColor: '#fff'
        })
      : mainWindow.setTitleBarOverlay({
          color: '#00000000',
          symbolColor: '#000'
        });
  }
};
