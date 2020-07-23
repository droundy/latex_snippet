from .latex_snippet import lib, ffi

def html(s):
    return ffi.string(lib.convert_html(s.encode())).decode()

def html_with_solution(s):
    return ffi.string(lib.latex_to_html_with_solution(s.encode())).decode()

def html_omit_solution(s):
    return ffi.string(lib.latex_to_html_omit_solution(s.encode())).decode()


def physics_macros(s):
    return ffi.string(lib.c_physics_macros(s.encode())).decode()

def omit_solutions(s):
    return ffi.string(lib.c_omit_solution(s.encode())).decode()
def omit_guide(s):
    return ffi.string(lib.c_omit_guide(s.encode())).decode()
def omit_handout(s):
    return ffi.string(lib.c_omit_handout(s.encode())).decode()
def only_handout(s):
    return ffi.string(lib.c_only_guide(s.encode())).decode()

