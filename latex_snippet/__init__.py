from .latex_snippet import lib, ffi

def html(s):
    return ffi.string(lib.convert_html(s.encode())).decode()
