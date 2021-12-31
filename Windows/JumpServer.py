# -*- coding: utf-8 -*-
import os
import re
import sys
import json
import base64
import platform
import subprocess

# 兼容python 2, 3
try:
    reload(sys)
    sys.setdefaultencoding('utf-8')
except Exception:
    pass

'''
{
    "filename": "{}-{}-jumpserver".format(username, name),
    "protocol": "ssh",
    "username": "laoguang",
    "token": "xxx",   // 认证 授权
    "config": "full address:s:rdjumpserver.fit2cloud.com:33390"
}

在Windows系统中，文件名命名规则如下：
　　1）文件名最长可以使用255个字符；
　　2）可以使用扩展名，扩展名用来表示文件类型，也可以使用多间隔符的扩展名（如win.ini.txt是一个合法的文件名，但其文件类型由最后一个扩展名决定）;
　　3）文件名中允许使用空格，但不允许使用下列字符（英文输入法状态）：< > / \ | : " * ?;
　　4）windows系统对文件名中字母的大小写在显示时有不同，但在使用时不区分大小写.
'''
BASE_DIR = os.path.dirname(__file__)


def remove_current_rdp_file():
    re_rdp_file = re.compile(r'.*\.rdp$')
    for filename in os.listdir(BASE_DIR):
        if re_rdp_file.search(filename):
            os.remove(os.path.join(BASE_DIR, filename))


class Rouse(object):
    def __init__(self, data):
        self.filename = self.get_filename(data)
        self.protocol = data['protocol']
        self.config = data['config']

    @staticmethod
    def get_filename(data):
        filename = data.get('filename', 'jms')
        if platform.system().lower() == 'windows':
            filename = re.sub(r'[<>/\\|:"*?]*', '', filename)
        return filename

    def handle_ssh(self):
        pass

    def handle_rdp(self):
        file_path = os.path.join(BASE_DIR, self.filename + '.rdp')
        with open(file_path, 'w') as f:
            f.write(self.config)
        if platform.system().lower() == 'windows':
            subprocess.call('mstsc.exe {0}'.format(file_path), shell=True)
        else:
            subprocess.call('open {0}'.format(file_path), shell=True)

    def run(self):
        getattr(self, 'handle_' + self.protocol)()


if __name__ == '__main__':
    args = sys.argv
    if len(args) != 1:
        d = args[1].replace('jms://', '')
        if d[-1] == '/':
            d = d[:-1]
        d = json.loads(base64.b64decode(d).decode())
        remove_current_rdp_file()
        instance = Rouse(d)
        instance.run()
