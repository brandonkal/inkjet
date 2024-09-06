#!/bin/bash
_inkjet_wrapper() {
    # Get dynamic completions and store in temporary file
    tempfile=/tmp/inkjet-dynamic-completions.bash
    inkjet inkjet-dynamic-completions bash >"$tempfile"
    # shellcheck disable=SC1090
    source "$tempfile"
    rm "$tempfile" >/dev/null 2>&1
    # Call the actual completion function
    _inkjet
}

_inkjet() {
    # This function will be dynamically replaced
    :
}

complete -F _inkjet_wrapper -o bashdefault -o default inkjet
