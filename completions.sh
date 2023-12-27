#/usr/bin/env bash
_mk_completions()
{
  COMPREPLY=($(compgen -W "$(mk --segments)" "${COMP_WORDS[1]}"))
}

complete -F _mk_completions mk