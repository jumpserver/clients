import log from 'electron-log';
import icon from '../../resources/JumpServer.ico?asset';

import * as fs from 'fs';
import * as path from 'path';

import { execFile } from 'child_process';
import { Conf, useConf } from 'electron-conf/main';
import { electronApp, is, optimizer } from '@electron-toolkit/utils';
import { app, BrowserWindow, ipcMain, session, shell } from 'electron';

process.env['ELECTRON_DISABLE_SECURITY_WARNINGS'] = 'true';

const defaults = {
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
let jms_sessionid = '';
let jms_csrftoken = '';

let openMainWindow = true;

// prettier-ignore
const platform = process.platform === 'win32' ? 'windows' : process.platform === 'darwin' ? 'macos' : 'linux';

let conf = new Conf({ defaults: defaults! });

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
  const sessionUrl = match ? match[1] : null;

  if (sessionUrl) {
    const decodedSessionJson = Buffer.from(sessionUrl, 'base64').toString('utf-8');

    try {
      const decodedSession = JSON.parse(decodedSessionJson);

      if (decodedSession.type === 'cookie') {
        openMainWindow = true;

        jms_sessionid = decodedSession.cookie.jms_sessionid;
        jms_csrftoken = decodedSession.cookie.jms_csrftoken;

        // 立即设置 cookie 到当前站点
        if (mainWindow && mainWindow.webContents.getURL()) {
          const currentUrl = mainWindow.webContents.getURL();
          const urlObj = new URL(currentUrl);
          const siteUrl = `${urlObj.protocol}//${urlObj.host}`;

          session.defaultSession.cookies.set({
            url: siteUrl,
            name: 'jms_sessionid',
            value: jms_sessionid,
            path: '/',
            httpOnly: false,
            secure: siteUrl.startsWith('https'),
            sameSite: 'no_restriction'
          });
          session.defaultSession.cookies.set({
            url: siteUrl,
            name: 'jms_csrftoken',
            value: jms_csrftoken,
            path: '/',
            httpOnly: false,
            secure: siteUrl.startsWith('https'),
            sameSite: 'no_restriction'
          });
        }

        mainWindow?.webContents.send('set-login-session', decodedSession.cookie.jms_sessionid);
        mainWindow?.webContents.send('set-login-csrfToken', decodedSession.cookie.jms_csrftoken);

        // 通知渲染进程设置 cookie
        mainWindow?.webContents.send('setup-cookies-for-site');
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

function updateUserConfigIfNeeded() {
  const userConfigPath = path.join(app.getPath('userData'), 'config.json');

  let subPath = path.join(process.resourcesPath);

  if (is.dev) {
    subPath = 'bin';
  }

  const defaultConfigPath = path.join(subPath, 'config.json');

  let userConfig: Record<string, any> = {};
  let defaultConfig: Record<string, any> = {};

  try {
    defaultConfig = JSON.parse(fs.readFileSync(defaultConfigPath, 'utf8'));
  } catch (err) {
    console.error('无法读取默认配置:', err);
    return;
  }

  if (!fs.existsSync(userConfigPath)) {
    // 初次运行，直接复制
    fs.copyFileSync(defaultConfigPath, userConfigPath);
    console.log('首次生成用户配置文件');
    return;
  }

  try {
    userConfig = JSON.parse(fs.readFileSync(userConfigPath, 'utf8'));
  } catch (err) {
    console.warn('用户配置读取失败，覆盖为默认配置');
    fs.copyFileSync(defaultConfigPath, userConfigPath);
    return;
  }

  const defaultVersion = defaultConfig.version || 1;
  const userVersion = userConfig.version || 1;

  if (defaultVersion > userVersion) {
    console.log(`配置文件版本更新：${userVersion} → ${defaultVersion}`);

    // 合并配置，保留用户其他字段，但强制覆盖关键字段
    const mergedConfig = {
      ...userConfig,
      ...defaultConfig,
      version: defaultVersion,
      protocol: defaultConfig.protocol,
      type: defaultConfig.type,
      arg_format: defaultConfig.arg_format,
      autoit: defaultConfig.autoit
    };

    try {
      fs.writeFileSync(userConfigPath, JSON.stringify(mergedConfig, null, 2), 'utf8');
      console.log('用户配置已更新');
    } catch (err) {
      console.error('写入用户配置失败:', err);
    }
    conf = new Conf({ defaults: JSON.parse(fs.readFileSync(userConfigPath, 'utf8')) });
  }
}

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

    // 添加允许跨域 cookie 的头
    headers!['Access-Control-Allow-Credentials'] = ['true'];

    callback({
      cancel: false,
      responseHeaders: headers
    });
  });

  session.defaultSession.webRequest.onBeforeSendHeaders((details, callback) => {
    callback({ requestHeaders: details.requestHeaders });
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

const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
  app.quit(); // ✅ 尽早退出，不执行后面的初始化
  process.exit(0);
}

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
// 🧠 在 app 准备前更新配置（需要先监听 'ready'，确保 app.getPath 可用）
app.once('ready', () => {
  updateUserConfigIfNeeded();
});

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

  log.info('whenReady openMainWindow: ', openMainWindow);

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
ipcMain.on('get-current-site', (_, site) => {
  if (jms_sessionid && jms_csrftoken) {
    session.defaultSession.cookies.set({
      url: site,
      name: 'jms_sessionid',
      value: jms_sessionid,
      path: '/',
      httpOnly: false,
      secure: site.startsWith('https'),
      sameSite: 'no_restriction'
    });
    session.defaultSession.cookies.set({
      url: site,
      name: 'jms_csrftoken',
      value: jms_csrftoken,
      path: '/',
      httpOnly: false,
      secure: site.startsWith('https'),
      sameSite: 'no_restriction'
    });
    console.log('Cookie 设置完成');
  } else {
    console.log('警告：jms_sessionid 或 jms_csrftoken 为空，无法设置 cookie');
  }
});

// 恢复保存的 cookie
ipcMain.on('restore-cookies', (_, { site, sessionId, csrfToken }) => {
  if (sessionId && csrfToken && site) {
    // 更新全局变量
    jms_sessionid = sessionId;
    jms_csrftoken = csrfToken;

    // 设置 cookie
    session.defaultSession.cookies.set({
      url: site,
      name: 'jms_sessionid',
      value: sessionId,
      path: '/',
      httpOnly: false,
      secure: site.startsWith('https'),
      sameSite: 'no_restriction'
    });
    session.defaultSession.cookies.set({
      url: site,
      name: 'jms_csrftoken',
      value: csrfToken,
      path: '/',
      httpOnly: false,
      secure: site.startsWith('https'),
      sameSite: 'no_restriction'
    });
    console.log('Cookie 恢复完成');
  } else {
    console.log('警告：恢复 cookie 参数不完整');
  }
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
