# -*- coding: utf-8 -*-

import os
import sys
import base64

if __name__ == '__main__':
    if len(sys.argv) != 1:
        r = sys.argv[1].split('//')[1]
        r = base64.b64decode(r)
        with open('C:\\tmp\\client.rdp', 'wb') as f:
            f.write(r)
        os.system('mstsc.exe C:\\tmp\\client.rdp')
