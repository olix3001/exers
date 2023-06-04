#!/bin/sh

# Script for running programs in a jail
# Usage: jail.sh <jail dir> <program> <...dependencies>
# Example: jail.sh /home/jail /bin/bash

# Create jail directory if it doesn't exist
mkdir -pv $1

# Create all necessary directories
mkdir -pv $1/etc
mkdir -pv $1/bin

# Copy all dependencies of the program (and their dependencies) to the jail
for dep in $(ldd $2 | grep -o '/.*/'); do
    mkdir -pv $1$dep
done
for dep in $(ldd $2 | grep -o '/\S*'); do
    cp -v $dep $1$(dirname $dep)
done

# Copy the program to the jail
cp -v $2 $1/bin

# Copy the program's dependencies to the jail
if [ $# -gt 2 ]; then
    for dep in ${@:3}; do
        cp -v $dep $1/bin
    done

    # Copy dependecies of the program's dependencies to the jail
    for dep in ${@:3}; do
        for dep2 in $(ldd $dep | grep -o '/.*/'); do
            mkdir -pv $1$dep2
        done
        for dep2 in $(ldd $dep | grep -o '/\S*'); do
            cp -v $dep2 $1$(dirname $dep2)
        done
    done
fi

# Run the program in the jail
sudo chroot $1 /bin/$(basename $2)

# Clean up
rm -rf $1