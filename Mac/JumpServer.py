# -*- coding: utf-8 -*-
import os
import re
import sys
import json
import base64
import platform
import subprocess

'''
{
    "filename": "{}-{}-jumpserver".format(username, name),
    "protocol": "ssh",
    "username": "laoguang",
    "token": "xxx",   // 认证 授权
    "config": "full address:s:rdjumpserver.fit2cloud.com:33390"
}
'''
BASE_DIR = os.path.dirname(__file__)


def remove_current_rdp_file():
    re_rdp_file = re.compile(r'.*\.rdp$')
    for filename in os.listdir(BASE_DIR):
        if re_rdp_file.search(filename):
            os.remove(os.path.join(filename))


class Rouse(object):
    def __init__(self, data):
        self.filename = data.get('filename', 'jms')
        self.protocol = data['protocol']
        self.config = data['config']

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
        d = json.loads(base64.b64decode(d).decode())
        remove_current_rdp_file()
        instance = Rouse(d)
        instance.run()
