"""Xplainit - Runtime code explanation for Python.

This package couples the compiled Rust extension (imported below via
``from .xplainit import *``) with the pure-Python helper subpackage
``xplainit.python`` (see :mod:`xplainit.python.tracer` and
:mod:`xplainit.python.decorators`).

Packaging both together is what makes the convenience API usable straight
from an installed wheel: ``xplainit.auto_trace()`` imports
``xplainit.python.tracer`` at runtime, so that subpackage must ship inside
the installed ``xplainit`` package (not merely live in the source tree).
"""

from .xplainit import *  # noqa: F401,F403  (compiled Rust extension)

__doc__ = xplainit.__doc__  # type: ignore[name-defined]  # noqa: F405
if hasattr(xplainit, "__all__"):  # noqa: F405
    __all__ = xplainit.__all__  # noqa: F405
