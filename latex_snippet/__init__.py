import ffi, lib

def html(s):
    return ffi.string(lib.convert_html(s.encode()))
