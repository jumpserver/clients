# -*- coding: utf-8 -*-
import os
import sys
import json
import base64
import platform

'''
{
 "protocol": "ssh",
 "username": "laoguang",
 "token": "xxx",   // 认证 授权
 "config": "full address:s:rdjumpserver.fit2cloud.com:33390"
}
'''
BASE_DIR = os.path.dirname(__file__)


class Rouse(object):
    def __init__(self, data):
        self.protocol = data['protocol']
        self.config = data['config']

    def handle_ssh(self):
        pass

    def handle_rdp(self):
        file_path = os.path.join(BASE_DIR, 'client.rdp')
        with open(file_path, 'w') as f:
            f.write(self.config)
        if platform.system().lower() == 'windows':
            os.system('mstsc.exe {0}'.format(file_path))
        else:
            os.system('open {0}'.format(file_path))

    def run(self):
        getattr(self, 'handle_' + self.protocol)()


if __name__ == '__main__':

    args = sys.argv
    if len(args) != 1:
        d = args[1].replace('jms://', '')
        d = json.loads(base64.b64decode(d).decode())
        instance = Rouse(d)
        instance.run()
