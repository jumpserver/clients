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

      // 添加调试：显示解析后的完整会话信息
      console.log('🔍 解析后的会话信息:', {
        type: decodedSession.type,
        site: decodedSession.site,
        cookie: {
          jms_sessionid: decodedSession.cookie?.jms_sessionid?.substring(0, 10) + '...',
          jms_csrftoken: decodedSession.cookie?.jms_csrftoken?.substring(0, 10) + '...'
        },
        rawData: decodedSessionJson.substring(0, 200) + '...'
      });

      if (decodedSession.type === 'cookie') {
        openMainWindow = true;

        jms_sessionid = decodedSession.cookie.jms_sessionid;
        jms_csrftoken = decodedSession.cookie.jms_csrftoken;

        // 立即设置 cookie 到当前站点
        if (mainWindow && mainWindow.webContents.getURL()) {
          const currentUrl = mainWindow.webContents.getURL();
          const urlObj = new URL(currentUrl);
          const siteUrl = `${urlObj.protocol}//${urlObj.host}`;

          const isSecure = siteUrl.startsWith('https');
          const cookieOptions = {
            path: '/',
            httpOnly: false,
            secure: isSecure,
            sameSite: (isSecure ? 'no_restriction' : 'lax') as 'no_restriction' | 'lax'
          };

          session.defaultSession.cookies
            .set({
              url: siteUrl,
              name: 'jms_sessionid',
              value: jms_sessionid,
              ...cookieOptions
            })
            .catch(error => {
              console.error('设置 jms_sessionid cookie 失败:', error);
            });

          session.defaultSession.cookies
            .set({
              url: siteUrl,
              name: 'jms_csrftoken',
              value: jms_csrftoken,
              ...cookieOptions
            })
            .catch(error => {
              console.error('设置 jms_csrftoken cookie 失败:', error);
            });
        }

        // 合并发送登录信息，确保 session 和 csrfToken 同时传递
        mainWindow?.webContents.send('set-login-credentials', {
          session: decodedSession.cookie.jms_sessionid,
          csrfToken: decodedSession.cookie.jms_csrftoken,
          site: decodedSession.site || 'https://jumpserver-test.cmdb.cc'
        });

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

  session.defaultSession.webRequest.onBeforeSendHeaders(async (details, callback) => {
    const requestHeaders = { ...details.requestHeaders };

    // 如果是对 JumpServer API 的请求，手动添加 cookie
    if (details.url.includes('/api/') && (jms_sessionid || jms_csrftoken)) {
      try {
        // 获取当前站点的 cookie
        const cookies = await session.defaultSession.cookies.get({
          url: details.url
        });

        const cookieStrings: string[] = [];
        for (const cookie of cookies) {
          if (cookie.name === 'jms_sessionid' || cookie.name === 'jms_csrftoken') {
            cookieStrings.push(`${cookie.name}=${cookie.value}`);
          }
        }

        if (cookieStrings.length > 0) {
          // 只设置我们的认证 cookie，不合并现有的
          const newCookieValue = cookieStrings.join('; ');

          (requestHeaders as any)['Cookie'] = newCookieValue;

          // 确保 CSRF token 也在请求头中
          if (jms_csrftoken) {
            (requestHeaders as any)['X-CSRFToken'] = jms_csrftoken;
          }

          // 修复跨域问题：设置正确的 Referer 和 Origin
          const urlObj = new URL(details.url);
          const origin = `${urlObj.protocol}//${urlObj.host}`;
          (requestHeaders as any)['Referer'] = origin + '/';
          (requestHeaders as any)['Origin'] = origin;

          // 对于用户 profile 请求，移除空的 X-JMS-ORG 头
          if (details.url.includes('/users/profile/') && requestHeaders['X-JMS-ORG'] === '') {
            delete requestHeaders['X-JMS-ORG'];
            console.log('🗑️ 移除空的 X-JMS-ORG 头');
          }

          // 修复 Sec-Fetch-Site 为 same-origin
          (requestHeaders as any)['Sec-Fetch-Site'] = 'same-origin';

          console.log('🍪 手动添加认证信息到请求头:', {
            Cookie: newCookieValue,
            'X-CSRFToken': jms_csrftoken ? jms_csrftoken.substring(0, 10) + '...' : 'none'
          });

          // 调试：打印完整的请求头信息
          console.log('🔍 完整请求头信息:', {
            url: details.url,
            method: details.method,
            headers: Object.keys(requestHeaders).reduce((acc, key) => {
              acc[key] =
                key.toLowerCase().includes('token') || key.toLowerCase().includes('cookie')
                  ? requestHeaders[key].substring(0, 20) + '...'
                  : requestHeaders[key];
              return acc;
            }, {} as any)
          });
        }
      } catch (error) {
        console.error('获取 cookie 失败:', error);
      }
    }

    callback({ requestHeaders });
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
ipcMain.on('get-current-site', async (_, site) => {
  const loginWindow = new BrowserWindow({
    width: 600,
    height: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true
    }
  });
  const loginUrl = `${site}/core/auth/login/?next=%2Fui%2F`;
  loginWindow.loadURL(loginUrl);
  loginWindow.webContents.on('will-redirect', (event, url) => {
    if (url.includes('/ui')) {
      // 表示登录完成，可以获取 token 或关闭窗口
      console.log('登录完成，获取 token 或执行其他操作', url);
      // cookies 中获取 jms_sessionid 和 jms_csrftoken
      session.defaultSession.cookies.get({}).then(cookies => {
        const csrfTokenCookie = cookies.find(cookie => cookie.name.includes('csrftoken'));
        const sessionIdCookie = cookies.find(cookie => cookie.name.includes('sessionid'));
        jms_csrftoken = csrfTokenCookie?.value || '';
        jms_sessionid = sessionIdCookie?.value || '';
        mainWindow?.webContents.send('set-login-cookies', {
          cookies: cookies,
          csrfToken: csrfTokenCookie?.value,
          site: site
        });
        loginWindow.close();
      });
    }
  });
});

// 恢复保存的 cookie
ipcMain.on('restore-cookies', async (_, { site, sessionId, csrfToken }) => {
  console.log('📨 收到 restore-cookies 请求:', {
    site,
    sessionId: sessionId?.substring(0, 10) + '...',
    csrfToken: csrfToken?.substring(0, 10) + '...'
  });

  // 添加调试：检查 URL 解析
  try {
    const urlObj = new URL(site);
    console.log('🔍 站点 URL 解析:', {
      origin: urlObj.origin,
      hostname: urlObj.hostname,
      protocol: urlObj.protocol,
      port: urlObj.port
    });
  } catch (error) {
    console.error('❌ URL 解析失败:', error);
  }

  if (sessionId && csrfToken && site) {
    try {
      // 先清理旧的同名 cookie
      const existingCookies = await session.defaultSession.cookies.get({
        url: site
      });

      for (const cookie of existingCookies) {
        if (cookie.name === 'jms_sessionid' || cookie.name === 'jms_csrftoken') {
          await session.defaultSession.cookies.remove(site, cookie.name);
          console.log('🗑️ 清理旧 cookie:', cookie.name);
        }
      }

      // 更新全局变量
      jms_sessionid = sessionId;
      jms_csrftoken = csrfToken;
      console.log('🔄 更新全局变量完成');

      // 设置新的 cookie
      const isSecure = site.startsWith('https');
      const cookieOptions = {
        path: '/',
        httpOnly: false,
        secure: isSecure,
        // 对于 HTTPS 使用 no_restriction，对于 HTTP 使用 lax
        sameSite: (isSecure ? 'no_restriction' : 'lax') as 'no_restriction' | 'lax'
      };

      await session.defaultSession.cookies.set({
        url: site,
        name: 'jms_sessionid',
        value: sessionId,
        ...cookieOptions
      });
      console.log('✅ 设置 jms_sessionid cookie 完成');

      await session.defaultSession.cookies.set({
        url: site,
        name: 'jms_csrftoken',
        value: csrfToken,
        ...cookieOptions
      });
      console.log('✅ 设置 jms_csrftoken cookie 完成');

      // 验证 cookie 是否设置成功
      const newCookies = await session.defaultSession.cookies.get({
        url: site
      });
      const sessionCookie = newCookies.find(c => c.name === 'jms_sessionid');
      const csrfCookie = newCookies.find(c => c.name === 'jms_csrftoken');
      console.log('🔍 验证 cookie 设置结果:', {
        sessionCookie: sessionCookie ? '✅ 存在' : '❌ 不存在',
        csrfCookie: csrfCookie ? '✅ 存在' : '❌ 不存在'
      });

      console.log('🎉 Cookie 恢复完成');

      // 通知渲染进程 cookie 设置完成
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send('cookies-restored', { success: true, site });
      }
    } catch (error) {
      console.error('❌ 恢复 cookie 失败:', error);

      // 通知渲染进程 cookie 设置失败
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send('cookies-restored', {
          success: false,
          error: error instanceof Error ? error.message : String(error),
          site
        });
      }
    }
  } else {
    console.log('⚠️ 警告：恢复 cookie 参数不完整:', {
      site,
      sessionId: !!sessionId,
      csrfToken: !!csrfToken
    });

    // 通知渲染进程参数不完整
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('cookies-restored', {
        success: false,
        error: '参数不完整',
        site
      });
    }
  }
});

// 清理特定站点的认证 cookie
ipcMain.on('clear-site-cookies', async (_, site) => {
  if (site) {
    try {
      const existingCookies = await session.defaultSession.cookies.get({
        url: site
      });

      for (const cookie of existingCookies) {
        if (cookie.name === 'jms_sessionid' || cookie.name === 'jms_csrftoken') {
          await session.defaultSession.cookies.remove(site, cookie.name);
        }
      }
      console.log('站点 Cookie 清理完成:', site);
    } catch (error) {
      console.error('清理站点 cookie 失败:', error);
    }
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
