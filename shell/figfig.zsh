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
typeset -gi _figfig_zed_fd=-1
typeset -g _figfig_zed_cursor=''
typeset -gF _figfig_zed_retry=0

_figfig_send() {
  (( _figfig_write_fd >= 0 )) || return 0
  # Six NUL-delimited fields, written using zsh builtins only.
  local detail=$(( _figfig_selected - 1 ))
  [[ $1 == complete ]] && detail=$_figfig_zed_cursor
  print -rn -u $_figfig_write_fd -- "$1"$'\0'"$_figfig_revision"$'\0'"$CURSOR"$'\0'"$PWD"$'\0'"$BUFFER"$'\0'"$detail"$'\0' 2>/dev/null || _figfig_disconnect
}

_figfig_zed_read_cursor() {
  emulate -L zsh
  local bytes=$1 chunk MATCH
  local -a match
  local -i MBEGIN MEND
  local -F deadline=$(( EPOCHREALTIME + 0.040 )) remaining
  # A device status reply marks the report so a modified F3 key (also ending
  # in CSI row;column R) on its own is always returned to ZLE.
  local report=$'\e\\[0n\e\\[([1-9][0-9]{0,3});([1-9][0-9]{0,3})R'
  while (( (remaining = deadline - EPOCHREALTIME) > 0 )) &&
        sysread -i $_figfig_zed_fd -s 256 -t "$remaining" chunk 2>/dev/null; do
    bytes+=$chunk
    if [[ $bytes =~ $report ]]; then
      _figfig_zed_cursor="$match[1];$match[2]"
      bytes="${bytes:0:$(( MBEGIN - 1 ))}${bytes:$MEND}"
      break
    fi
  done
  # Preserve input that arrived during the query, including split replies.
  if [[ -z $_figfig_zed_cursor && -n $1 ]]; then
    bytes=${bytes#"$1"}
  fi
  [[ -z $bytes ]] || zle -U -- "$bytes"
  [[ -n $_figfig_zed_cursor ]]
}

_figfig_zed_late_reply() {
  local _figfig_zed_cursor=''
  # The status marker was already consumed by ZLE. If the terminal missed the
  # deadline, discard its report without changing the command or its popup.
  _figfig_zed_read_cursor $'\e[0n' || return 0
}

_figfig_zed_query_cursor() {
  _figfig_zed_cursor=''
  (( _figfig_zed_fd >= 0 && _figfig_write_fd >= 0 && EPOCHREALTIME >= _figfig_zed_retry )) || return 0
  zle -R
  print -rn -- $'\e[5n\e[6n' >/dev/tty 2>/dev/null || return 0
  _figfig_zed_read_cursor '' || _figfig_zed_retry=$(( EPOCHREALTIME + 1 ))
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
  # Hide the helper's job/PID announcement, restoring job control on return.
  setopt localoptions nomonitor
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
  region_highlight=( "${(@)region_highlight:#*memo=figfig-history}" )
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
      region_highlight+=( "${#BUFFER} $(( ${#BUFFER} + ${#POSTDISPLAY} )) fg=242 memo=figfig-history" )
      break
    fi
  done
}

_figfig_redraw() {
  _figfig_history
  if [[ $BUFFER != $_figfig_last_buffer || $CURSOR != $_figfig_last_cursor ]]; then
    _figfig_visible=0
    # Let ZLE finish processing a burst of keys before asking for geometry.
    (( _figfig_zed_fd >= 0 && (PENDING || KEYS_QUEUED_COUNT) )) && return 0
    _figfig_last_buffer=$BUFFER
    _figfig_last_cursor=$CURSOR
    if [[ $BUFFER != $_figfig_suppressed ]]; then
      _figfig_suppressed=''
      (( ++_figfig_revision ))
      if [[ -n $BUFFER && $KEYMAP != vicmd ]]; then
        _figfig_zed_query_cursor
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
if [[ $OSTYPE == darwin* && ${TERM_PROGRAM-} == zed ]] &&
   sysopen -r -o cloexec -u _figfig_zed_fd /dev/tty 2>/dev/null; then
  zle -N _figfig_zed_late_reply
  for _figfig_map in emacs viins vicmd; do
    bindkey -M $_figfig_map $'\e[0n' _figfig_zed_late_reply
  done
  unset _figfig_map
fi
add-zle-hook-widget line-init _figfig_line_init
add-zle-hook-widget line-pre-redraw _figfig_redraw
add-zle-hook-widget line-finish _figfig_line_finish
add-zsh-hook zshexit _figfig_disconnect
