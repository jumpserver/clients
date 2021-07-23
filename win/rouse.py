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
    # d = 'eyJwcm90b2NvbCI6ICJyZHAiLCAidXNlcm5hbWUiOiAibGFvZ3VhbmciLCAidG9rZW4iOiAieHh4IiwgImNvbmZpZyI6ICJmdWxsIGFkZHJlc3M6czp5eS5maXQyY2xvdWQuY29tOjMzMzg5XG51c2VybmFtZTpzOmZlbmdxaWFuZ3xTNEY2RUZzTVo3TjZVTVBZcFRWdzJQSFVSZ3dKVkpNcHAzZEZcbnNjcmVlbiBtb2RlIGlkOmk6MFxudXNlIG11bHRpbW9uOmk6MVxuc2Vzc2lvbiBicHA6aTozMlxuYXVkaW9tb2RlOmk6MFxuZGlzYWJsZSB3YWxscGFwZXI6aTowXG5kaXNhYmxlIGZ1bGwgd2luZG93IGRyYWc6aTowXG5kaXNhYmxlIG1lbnUgYW5pbXM6aTowXG5kaXNhYmxlIHRoZW1lczppOjBcbmFsdGVybmF0ZSBzaGVsbDpzOlxuc2hlbGwgd29ya2luZyBkaXJlY3Rvcnk6czpcbmF1dGhlbnRpY2F0aW9uIGxldmVsOmk6MlxuY29ubmVjdCB0byBjb25zb2xlOmk6MFxuZGlzYWJsZSBjdXJzb3Igc2V0dGluZzppOjBcbmFsbG93IGZvbnQgc21vb3RoaW5nOmk6MVxuYWxsb3cgZGVza3RvcCBjb21wb3NpdGlvbjppOjFcbnJlZGlyZWN0cHJpbnRlcnM6aTowXG5wcm9tcHQgZm9yIGNyZWRlbnRpYWxzIG9uIGNsaWVudDppOjBcbmF1dG9yZWNvbm5lY3Rpb24gZW5hYmxlZDppOjFcbmJvb2ttYXJrdHlwZTppOjNcbnVzZSByZWRpcmVjdGlvbiBzZXJ2ZXIgbmFtZTppOjBcbnNtYXJ0IHNpemluZzppOjEifQ=='

    args = sys.argv
    if len(args) != 1:
        d = args[1].replace('jms://', '')
        d = json.loads(base64.b64decode(d).decode())
        instance = Rouse(d)
        instance.run()
