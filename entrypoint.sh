#!/usr/bin/env bash 
set -euo pipefail

printf "\n=====\n - STARTING API - \n=====\n"

/usr/local/bin/piss-fan \
    --login "$USER_LOGIN" \
    --app-token "$APP_TOKEN" \
    --user-token "$USER_TOKEN"

