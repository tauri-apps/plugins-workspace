#!/bin/bash

if git diff --quiet --ignore-submodules HEAD
then
  echo "working directory is clean"
else
  echo "found diff"
  git diff --name-status HEAD
  exit 1
fi
