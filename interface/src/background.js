import {app, BrowserWindow, ipcMain, protocol} from "electron";
import {createProtocol} from "vue-cli-plugin-electron-builder/lib";
import installExtension, {VUEJS3_DEVTOOLS} from "electron-devtools-installer";
import path from 'path'
import fse from 'fs-extra'
import {execFile} from "child_process";

app.commandLine.appendSwitch('disable-gpu-sandbox');

const isDevelopment = process.env.NODE_ENV !== "production";
// Scheme must be registered before the app is ready
protocol.registerSchemesAsPrivileged([
    {scheme: "app", privileges: {secure: true, standard: true}},
]);

let mainWindow

async function createWindow() {
    // Create the browser window.
    mainWindow = new BrowserWindow({
        width: 860,
        height: 550,
        center: true,
        fullscreenable: false,
        resizable: false,
        transparent: true,
        titleBarStyle: 'hidden',
        titleBarOverlay: {
            color: "#1f1f1f",
            symbolColor: "#fff",
        },
        webPreferences: {
            // Use pluginOptions.nodeIntegration, leave this alone
            // See nklayman.github.io/vue-cli-plugin-electron-builder/guide/security.html#node-integration for more info
            nodeIntegration: true,
            contextIsolation: false,
        },
    });

    if (process.env.WEBPACK_DEV_SERVER_URL) {
        // Load the url of the dev server if in development mode
        await mainWindow.loadURL(process.env.WEBPACK_DEV_SERVER_URL);
        // if (!process.env.IS_TEST) mainWindow.webContents.openDevTools()
    } else {
        createProtocol("app");
        // Load the index.html when not in development
        await mainWindow.loadURL("app://./index.html");
        // mainWindow.webContents.openDevTools()
    }
}

// Quit when all windows are closed.
app.on("window-all-closed", () => {
    // On macOS it is common for applications and their menu bar
    // to stay active until the user quits explicitly with Cmd + Q
    if (process.platform !== "darwin") {
        app.quit();
    }
});

app.on("activate", () => {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
});

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on("ready", async () => {
    if (isDevelopment && !process.env.IS_TEST) {
        // Install Vue Devtools
        try {
            await installExtension(VUEJS3_DEVTOOLS);
        } catch (e) {
            console.error("Vue Devtools failed to install:", e.toString());
        }
    }
    registerLocalResourceProtocol();
    createWindow();
});

function registerLocalResourceProtocol() {
    protocol.registerFileProtocol("local-resource", (request, callback) => {
        const url = request.url.replace(/^local-resource:\/\//, "");
        // Decode URL to prevent errors when loading filenames with UTF-8 chars or chars like "#"
        const decodedUrl = decodeURI(url); // Needed in case URL contains spaces
        try {
            return callback(decodedUrl);
        } catch (error) {
            console.error(
                "ERROR: registerLocalResourceProtocol: Could not get file path:",
                error
            );
        }
    });
}

// Exit cleanly on request from parent process in development mode.
if (isDevelopment) {
    if (process.platform === "win32") {
        process.on("message", (data) => {
            if (data === "graceful-exit") {
                app.quit();
            }
        });
    } else {
        process.on("SIGTERM", () => {
            app.quit();
        });
    }
}


if (process.platform === "darwin") {
  if (process.defaultApp) {
    if (process.argv.length >= 2) {
      app.setAsDefaultProtocolClient('jms', process.execPath, [path.resolve(process.argv[1])])
    }
  } else {
    app.setAsDefaultProtocolClient('jms')
  }
}

const handleOpenFromUrl = (url) => {
    let subPath
    if (isDevelopment && !process.env.IS_TEST) {
        subPath = "bin"
    } else {
        subPath = process.resourcesPath + "/bin"
    }
    if (process.platform === "linux") {
        switch (process.arch) {
            case 'x32':
            case 'x64':
                subPath += "/linux-amd64"
                break;
            case 'arm':
            case 'arm64':
                subPath += "/linux-arm64"
                break;
        }
    } else if (process.platform === "darwin") {
        subPath += "/darwin"
    }
    console.log(subPath)
    let exeFilePath = path.join(subPath, 'JumpServerClient')
    const {execFile} = require('child_process')
    execFile(exeFilePath, [url], (error, stdout, stderr) => {
        if (error) {
            console.log(error);
        }
    });
}

app.on('open-url', (event, urlStr) => {
    handleOpenFromUrl(urlStr);
});

// 隐藏主窗口
const hideWindow = () => {
    if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.hide()
    }
}

let configFilePath

let subPath
if (isDevelopment && !process.env.IS_TEST) {
    subPath = "bin"
} else {
    subPath = process.resourcesPath + "/bin"
}
configFilePath = path.join(subPath, 'config.json')


const callBackUrlName = 'config-reply-get'//消息发布-发布名称
//读取本地文件
ipcMain.on('config-get', function (event) {
    // 传给渲染进程数据
    fse.readFile(configFilePath, "utf8", (err, data) => {
        if (err) {
            event.sender.send(callBackUrlName, 500, "读取 config.json 文件失败");
        } else {
            event.sender.send(callBackUrlName, 200, data);
        }
    })
});


//增改本地文件
ipcMain.on('config-set', function (event, type, value) {
    value = JSON.parse(value)
    fse.readFile(configFilePath, "utf8", (err, data) => {
        if (err) {
            console.log("目标文件异常")
        } else {
            let config = JSON.parse(data);
            let platform
            if (process.platform === "win32") {
                platform = "windows"
            } else if (process.platform === "darwin") {
                platform = "macos"
            } else {
                platform = "linux"
            }
            let lst = []
            switch (type) {
                case 'sshPage':
                    lst = config[platform]['terminal']
                    break
                case 'remotePage':
                    lst = config[platform]['remotedesktop']
                    break
                case 'fileTransferPage':
                    lst = config[platform]['filetransfer']
                    break
                case 'databasesPage':
                    lst = config[platform]['databases']
                    break
            }
            lst.forEach(item => {
                if (value.is_default) {
                    item.is_default = false
                }
                if (item.match_first.length > 0) {
                    item.match_first = item.match_first.filter(item => !value.match_first.includes(item))
                }
                if (item.name === value.name) {
                    item.path = value.path
                    item.is_default = value.is_default
                    item.is_set = value.is_set
                    item.match_first = value.match_first
                }
            })
            const config_str = JSON.stringify(config)
            fse.writeFileSync(configFilePath, config_str, "utf8")
            event.sender.send(callBackUrlName, 200, config_str);
        }
    })
});
