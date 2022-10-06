#!/bin/bash

SETUP_ENV=/env/setup_env.sh

if [[ -f "$SETUP_ENV" ]]; then
    source $SETUP_ENV
fi

if [[ -z "${GIT_USERNAME}" ]]; then
  echo "Please set GIT_USERNAME environment variable"
  exit 1
fi

if [[ -z "${GIT_USEREMAIL}" ]]; then
  echo "Please set GIT_USEREMAIL enviroment variable"
  exit 1
fi

git config --global user.email "${GIT_USEREMAIL}"
git config --global user.name "${GIT_USERNAME}"
git config --global core.eol lf
git config --global core.autocrlf false
git config --global pull.rebase true
git config --global init.defaultBranch master

/app/webhook
