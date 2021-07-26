# Apps

## Install

### Mac

下载Mac安装包，安装JumpServer.dmg并将JumpServer.app放入Applications中，双击打开JumpServer.app完成自定义协议。

### Win

下载Win安装包，双击JumpServer.msi 文件（需要等待10几秒钟）。

## Uninstall

### Windows

控制面板直接卸载即可。

### Mac

移除JumpServer.app。

## 测试

url link: jms://eyJwcm90b2NvbCI6ICJyZHAiLCAidXNlcm5hbWUiOiAibGFvZ3VhbmciLCAidG9rZW4iOiAieHh4IiwgImNvbmZpZyI6ICJmdWxsIGFkZHJlc3M6czp5eS5maXQyY2xvdWQuY29tOjMzMzg5XG51c2VybmFtZTpzOmZlbmdxaWFuZ3xTNEY2RUZzTVo3TjZVTVBZcFRWdzJQSFVSZ3dKVkpNcHAzZEZcbnNjcmVlbiBtb2RlIGlkOmk6MFxudXNlIG11bHRpbW9uOmk6MVxuc2Vzc2lvbiBicHA6aTozMlxuYXVkaW9tb2RlOmk6MFxuZGlzYWJsZSB3YWxscGFwZXI6aTowXG5kaXNhYmxlIGZ1bGwgd2luZG93IGRyYWc6aTowXG5kaXNhYmxlIG1lbnUgYW5pbXM6aTowXG5kaXNhYmxlIHRoZW1lczppOjBcbmFsdGVybmF0ZSBzaGVsbDpzOlxuc2hlbGwgd29ya2luZyBkaXJlY3Rvcnk6czpcbmF1dGhlbnRpY2F0aW9uIGxldmVsOmk6MlxuY29ubmVjdCB0byBjb25zb2xlOmk6MFxuZGlzYWJsZSBjdXJzb3Igc2V0dGluZzppOjBcbmFsbG93IGZvbnQgc21vb3RoaW5nOmk6MVxuYWxsb3cgZGVza3RvcCBjb21wb3NpdGlvbjppOjFcbnJlZGlyZWN0cHJpbnRlcnM6aTowXG5wcm9tcHQgZm9yIGNyZWRlbnRpYWxzIG9uIGNsaWVudDppOjBcbmF1dG9yZWNvbm5lY3Rpb24gZW5hYmxlZDppOjFcbmJvb2ttYXJrdHlwZTppOjNcbnVzZSByZWRpcmVjdGlvbiBzZXJ2ZXIgbmFtZTppOjBcbnNtYXJ0IHNpemluZzppOjEifQ==

## Packaging

### Mac

- 双击打开JumpServer.scpt文件。

- 点击 文件 --> 导出（导出为应用程序）![image-20210726112425650](/Users/fengqiang/Desktop/mac-app.png) 

- 当前目录上会生成一个JumpServer.app 应用程序。（打开JumpServer.app包 前往 Contents --> Resources 去更换applet.icns图标）。

- 打开JumpServer.app包 将JumpServer.py放入包中（下个版本JumpServer.py 将转换为golang语言编写 打包成二进制文件）

- 前往JumpServer.app包 --> Contents 修改Info.plist文件，添加代码如下：

  ```
  <key>CFBundleURLTypes</key>
    <array>
      <dict>
        <key>CFBundleURLName</key>
          <string>Jms</string>
          <key>CFBundleURLSchemes</key>
          <array>
            <string>jms</string>
          </array>
      </dict>
    </array>
  ```

- 选择Mac自带的磁盘工具 选择文件 --> 新建映像 --> 空白映像 存储。<img src="/Users/fengqiang/Desktop/browser-tool/static/img/空白映像.png" alt="空白映像" style="zoom:50%;" />

- 打开映盘文件 将app文件及Applications的快捷方式放入（可修改背景）。<img src="/Users/fengqiang/Desktop/browser-tool/static/img/打包dmg.png" alt="打包dmg" style="zoom:50%;" />

- 推出磁盘映像。
- 最后通过磁盘工具 选择 映像 --> 转换 --> 选取文件后 --> 映像格式选压缩 --> 点击转换。（完成）

###  Windows



