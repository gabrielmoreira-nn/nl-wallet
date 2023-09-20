#!/usr/bin/env bash

export SCRIPTS_DIR=$(dirname $(realpath $(command -v ${BASH_SOURCE[0]})))

source "${SCRIPTS_DIR}/colors.sh"

if command -v adb > /dev/null
then
    echo -e "Mapping Android ports with ${GREEN}adb${NC}"
    if adb reverse tcp:3000 tcp:3000
    then
        adb reverse tcp:3003 tcp:3003
        adb reverse tcp:8006 tcp:8006
    else
        echo -e "Please start the Android emulator, and run ${BLUE}$0 $@${NC} again"
    fi
else
    echo -e "Android ${GREEN}adb${NC} command not found"
fi