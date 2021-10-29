#!/bin/sh

echo 'Deleting possible old man pages...'
rm _build/man/*.1
echo 'Build man page...'
make man
echo 'Opening man page'
man ./_build/man/dmdt.1
echo 'Deleting man page'
rm _build/man/*
