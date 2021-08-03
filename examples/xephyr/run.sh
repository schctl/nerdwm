XEPHYR=$(which Xephyr)
ABS_PATH=$(dirname $(realpath $0))

xinit $ABS_PATH/xinitrc -- \
    "$XEPHYR" \
        :100 \
        -ac \
        -screen 800x600 \
        -host-cursor
