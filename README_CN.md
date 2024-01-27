# Jumpserve Client

JumpServer 客户端，支持本地本地客户端拉起。

中文 | [English](README.md)

## 概览

![windows](static/windows_CN.png)

## 安装

### Mac

下载Mac安装包，安装JumpServer.dmg并将JumpServer.app放入Applications中，双击打开JumpServer.app完成自定义协议。

### Win

下载Win安装包，双击JumpServer安装包文件（需要等待10几秒钟）。

### Linux

下载Linux安装包，双击Deb安装包文件，打开安装包管理器进行安装。或者使用命令进行安装
```
sudo dpkg -i <package_file>
```


## 卸载

### Windows

控制面板直接卸载即可。

### Mac

移除JumpServer.app。

### Linux

点击打开软件管理器进行卸载。或者使用命令进行卸载
```
sudo dpkg --purge <package_file>
```

## 打包

- [打包方法](https://github.com/jumpserver/apps/blob/master/README_PACK.md)

## 参考

- [浏览器拉起app原理](https://juejin.cn/post/6844903989155217421)
- [Linux浏览器拉起app原理](https://medium.com/swlh/custom-protocol-handling-how-to-8ac41ff651eb)
- [刘正的JumpClient](https://github.com/liuzheng/jumpClient)
