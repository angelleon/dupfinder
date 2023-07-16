#!/bin/bash

OPS=$(getopt \
--options v:,n:,a,A,r,d,t,s,w,D \
--longoptions version:,name:,arch,all-packages,rpm,deb,tarball,source,win,darwin \
--name package -- "$@")

eval set -- $OPS

VERSION='v0.1'
NAME='dupfinder'
ALL_PACKAGES=( arch deb rpm source tarball win darwin )
TARGET_PACKAGES=''


while true;
do
    # echo $@
    case "$1" in
        -v | --version )
            VERSION="$2"
            shift 2
        ;;
        -n | --name)
            NAME="$2"
            shift 2
        ;;
        -a | --all-packages )
            TARGET_PACKAGES=$ALL_PACKAGES
            shift 1
        ;;
        -t | --tarball )
            TARGET_PACKAGES+=(tarball)
            shift 1
        ;;
        *)
            break
        ;;
    esac
done


for target in ${TARGET_PACKAGES[@]}
do
    echo packaging $target
    unset -f _package_main 2>/dev/null || true
    source package/package-$target.sh
    _package_main $NAME-$VERSION
done