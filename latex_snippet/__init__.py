from .latex_snippet import lib, ffi

def html(s):
    return ffi.string(lib.convert_html(s.encode())).decode()

def html_with_solution(s):
    return ffi.string(lib.latex_to_html_with_solution(s.encode())).decode()

def html_omit_solution(s):
    return ffi.string(lib.latex_to_html_omit_solution(s.encode())).decode()
