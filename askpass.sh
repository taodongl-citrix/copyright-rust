#!/bin/bash
case "$1" in
  Username*) echo "${GIT_USERNAME}" ;;
  Password*) echo "${GIT_PASSWORD}" ;;
esac