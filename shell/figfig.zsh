# figfig's ZLE integration. Source after other shell plugins.
[[ -o interactive && -o zle ]] || return 0
[[ -z ${SSH_CONNECTION-}${TMUX-} ]] || return 0
[[ -z ${_FIGFIG_LOADED-} ]] || return 0
typeset -g _FIGFIG_LOADED=1
zmodload zsh/system
zmodload zsh/datetime
zmodload zsh/parameter
autoload -Uz add-zle-hook-widget add-zsh-hook

typeset -g _figfig_session="$$-${EPOCHREALTIME}"
typeset -gi _figfig_revision=0 _figfig_visible=0 _figfig_selected=1
typeset -gi _figfig_last_cursor=-1
typeset -gi _figfig_start=0 _figfig_end=0 _figfig_read_fd=-1 _figfig_write_fd=-1
typeset -g _figfig_last_buffer='' _figfig_suppressed='' _figfig_helper_pid=''
typeset -ga _figfig_insert=() _figfig_space=()
typeset -gA _figfig_original=()

_figfig_send() {
  (( _figfig_write_fd >= 0 )) || return 0
  # Six NUL-delimited fields, written using zsh builtins only.
  print -rn -u $_figfig_write_fd -- "$1"$'\0'"$_figfig_revision"$'\0'"$CURSOR"$'\0'"$PWD"$'\0'"$BUFFER"$'\0'"$(( _figfig_selected - 1 ))"$'\0' 2>/dev/null || _figfig_disconnect
}

_figfig_disconnect() {
  if (( _figfig_read_fd >= 0 )); then
    zle -F $_figfig_read_fd 2>/dev/null
    exec {_figfig_read_fd}<&-
  fi
  if (( _figfig_write_fd >= 0 )); then
    exec {_figfig_write_fd}>&-
  fi
  _figfig_read_fd=-1 _figfig_write_fd=-1 _figfig_visible=0
  _figfig_insert=() _figfig_space=()
}

_figfig_connect() {
  (( _figfig_read_fd < 0 )) || return 0
  # Process substitution provides a persistent, bidirectional helper pipe.
  local helper_in helper_out
  coproc command "$_FIGFIG_BIN" bridge "$_figfig_session" 2>/dev/null
  _figfig_helper_pid=$!
  exec {_figfig_write_fd}>&p
  exec {_figfig_read_fd}<&p
  zle -N _figfig_receive
  zle -F -w $_figfig_read_fd _figfig_receive
}

_figfig_receive() {
  if [[ -n ${2-} ]]; then _figfig_disconnect; return 0; fi
  local line
  if ! IFS= read -r -u $_figfig_read_fd line; then _figfig_disconnect; return 0; fi
  local -a fields=( ${(z)line} )
  if [[ ${fields[1]-} == P ]]; then
    typeset -g FIGFIG_HISTORY_ENABLED=${fields[2]:-1}
    _figfig_history
    zle -R
    return 0
  fi
  [[ ${fields[1]-} == R && ${fields[2]-} == $_figfig_revision ]] || return 0
  [[ $BUFFER != $_figfig_suppressed ]] || return 0
  _figfig_start=${fields[3]} _figfig_end=${fields[4]} _figfig_visible=${fields[5]}
  _figfig_insert=() _figfig_space=()
  local -i count=${fields[6]:-0} i
  (( count <= 12 && ${#fields} == 6 + count * 2 )) || { _figfig_visible=0; return 0; }
  for (( i=1; i<=count; ++i )); do
    _figfig_insert+=( "${(Q)fields[$(( 5 + i * 2 ))]}" )
    _figfig_space+=( "${fields[$(( 6 + i * 2 ))]}" )
  done
  _figfig_selected=1
  (( count > 0 )) || _figfig_visible=0
  zle -R
}

_figfig_hide() {
  _figfig_visible=0
  _figfig_insert=() _figfig_space=()
  (( ++_figfig_revision ))
  _figfig_send hide
}

_figfig_history() {
  POSTDISPLAY=''
  # Keep history inside the shell. Never copy the history file to the app.
  [[ ${FIGFIG_HISTORY_ENABLED:-1} == 1 && -n $BUFFER && $CURSOR == ${#BUFFER} && $KEYMAP != vicmd ]] || return 0
  local key line
  local -i checked=0
  for key in ${(Onk)history}; do
    (( ++checked > 2000 )) && break
    line=${history[$key]}
    if [[ $line == "$BUFFER"* && $line != "$BUFFER" && $line != *$'\n'* ]]; then
      POSTDISPLAY=${line[$(( ${#BUFFER} + 1 )),-1]}
      break
    fi
  done
}

_figfig_redraw() {
  _figfig_history
  if [[ $BUFFER != $_figfig_last_buffer || $CURSOR != $_figfig_last_cursor ]]; then
    _figfig_last_buffer=$BUFFER
    _figfig_last_cursor=$CURSOR
    _figfig_visible=0
    if [[ $BUFFER != $_figfig_suppressed ]]; then
      _figfig_suppressed=''
      (( ++_figfig_revision ))
      if [[ -n $BUFFER && $KEYMAP != vicmd ]]; then
        _figfig_send complete
      else
        _figfig_send hide
      fi
    fi
  fi
}

_figfig_line_init() {
  _figfig_last_buffer='' _figfig_suppressed=''
  _figfig_connect
}

_figfig_line_finish() {
  POSTDISPLAY=''
  _figfig_hide
}

_figfig_accept() {
  (( _figfig_visible && ${#_figfig_insert} )) || return 1
  local left='' right='' insert=${_figfig_insert[$_figfig_selected]}
  (( _figfig_start > 0 )) && left=${BUFFER[1,$_figfig_start]}
  (( _figfig_end < ${#BUFFER} )) && right=${BUFFER[$(( _figfig_end + 1 )),-1]}
  [[ ${_figfig_space[$_figfig_selected]} == 1 && $right != ' '* ]] && insert+=' '
  BUFFER="$left$insert$right"
  CURSOR=$(( ${#left} + ${#insert} ))
  POSTDISPLAY=''
  _figfig_suppressed=$BUFFER _figfig_last_buffer=$BUFFER
  _figfig_hide
  return 0
}

_figfig_key() {
  local action=$1
  shift
  case $action in
    accept)
      _figfig_accept && return 0
      ;;
    up|down)
      if (( _figfig_visible && ${#_figfig_insert} )); then
        if [[ $action == up ]]; then
          (( _figfig_selected = (_figfig_selected + ${#_figfig_insert} - 2) % ${#_figfig_insert} + 1 ))
        else
          (( _figfig_selected = _figfig_selected % ${#_figfig_insert} + 1 ))
        fi
        _figfig_send select
        return 0
      fi
      ;;
    right)
      if [[ -n $POSTDISPLAY && $CURSOR == ${#BUFFER} ]]; then
        BUFFER+=$POSTDISPLAY
        CURSOR=${#BUFFER}
        POSTDISPLAY=''
        _figfig_suppressed=$BUFFER _figfig_last_buffer=$BUFFER
        _figfig_hide
        return 0
      fi
      ;;
    escape)
      if (( _figfig_visible )); then
        _figfig_suppressed=$BUFFER
        _figfig_hide
        return 0
      fi
      ;;
  esac
  local original=${_figfig_original[$WIDGET]-}
  [[ -n $original ]] && zle "$original" -- "$@"
  return 0
}

# Wrap the actual key bindings, preserving user/Oh My Zsh widgets.
_figfig_bind() {
  local map=$1 key=$2 action=$3 fallback=$4 binding original widget
  binding=$(bindkey -M "$map" "$key" 2>/dev/null)
  original=${${(z)binding}[-1]}
  original=${(Q)original}
  [[ -n $original && $original != undefined-key ]] || original=$fallback
  widget="figfig-${map}-${action}-${#_figfig_original}"
  zle -A "$original" "${widget}-original" 2>/dev/null || return 0
  _figfig_original[$widget]="${widget}-original"
  functions[$widget]="_figfig_key $action \"\$@\""
  zle -N "$widget"
  bindkey -M "$map" "$key" "$widget"
}

for _figfig_map in emacs viins; do
  _figfig_bind $_figfig_map '^I' accept expand-or-complete
  _figfig_bind $_figfig_map '^M' accept accept-line
  _figfig_bind $_figfig_map '^J' accept accept-line
  _figfig_bind $_figfig_map '^[[A' up up-line-or-history
  _figfig_bind $_figfig_map '^[[B' down down-line-or-history
  _figfig_bind $_figfig_map '^[OA' up up-line-or-history
  _figfig_bind $_figfig_map '^[OB' down down-line-or-history
  _figfig_bind $_figfig_map '^[[C' right forward-char
  _figfig_bind $_figfig_map '^[OC' right forward-char
  _figfig_bind $_figfig_map '^[' escape send-break
done
unset _figfig_map
zle_highlight+=( 'suffix:fg=8' )
add-zle-hook-widget line-init _figfig_line_init
add-zle-hook-widget line-pre-redraw _figfig_redraw
add-zle-hook-widget line-finish _figfig_line_finish
add-zsh-hook zshexit _figfig_disconnect
