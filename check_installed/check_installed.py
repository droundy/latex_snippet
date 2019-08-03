#!/usr/bin/env python3

import latex_snippet

print(dir(latex_snippet))

from latex_snippet import html

assert html('foo \it bar') == 'foo <em>bar</em>'

print("SUCCESS")
