#!/bin/bash

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

# Copy additional files to the jail
if [ $# -gt 2 ]; then
    for file in ${@:3}; do
        cp -v $file $1/$file
    done
fi

# Run the program in the jail
sudo chroot $1 /bin/$(basename $2) ${@:3}

# Clean up
rm -rf $1