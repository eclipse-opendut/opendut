#!/bin/bash

###############################################################################
LOG_DIR=/tmp/xvfb-firefox/
XVFB_DISPLAY_SIZE="1920x1080x24"
X11VNC_DISPLAY_SIZE="1920x1080x24"
X_RESIZE="1910 1070"
ARG="$1"
export DISPLAY=":1"
mkdir -p $LOG_DIR
###############################################################################
resize_firefox_window() {
  WINDOW_IDS=$(xdotool search --onlyvisible firefox 2>/dev/null)
  for ID in $WINDOW_IDS
  do
    xdotool windowmove "$ID" 0 0
    # shellcheck disable=SC2086
    xdotool windowsize "$ID" $X_RESIZE
  done
}

start_framebuffer() {
    Xvfb -screen 0 "$XVFB_DISPLAY_SIZE" "$DISPLAY" > "$LOG_DIR"/xvfb.log 2>&1 &
    x11vnc -display "$DISPLAY" -geometry "$X11VNC_DISPLAY_SIZE" -repeat -noxdamage -ncache 10 -forever -loop -o "$LOG_DIR"/x11vnc-program.log > "$LOG_DIR"/x11vnc-stdout.log 2>&1 &
}

start_firefox() {
  # firefox in framebuffer
  firefox --display="$DISPLAY" > "$LOG_DIR"/firefox.log 2>&1

  echo "Wait for firefox startup"
  sleep 5
  resize_firefox_window
}

stop_firefox() {
  killall firefox
  killall Xvfb
  killall x11vnc
}

###############################################################################


case $ARG in
start)
  start_firefox;;
firefox)
  start_firefox;;
stop)
  stop_firefox;;
resize)
  resize_firefox_window;;
*)
  echo "Unknown argument!";;
esac

