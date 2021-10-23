#!/bin/sh

make man
man ./_build/man/devmode.1
rm _build/man/*
