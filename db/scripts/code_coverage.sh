#!/usr/bin/env bash
set -e

source_root_dir="db/"
local_report_dir="report/"
build_dir="target/debug/"
#report_dir="db/report/"
report_dir="$source_root_dir$local_report_dir"

echo "Check if in correct directory":
path=$(pwd)
primary_dir=$(basename $path)
if [ "$primary_dir" == "db" ]; then
    echo "    + Moving to correct directory.."
    cd ..
elif [ "$primary_dir" == "scripts" ]; then
    echo "    + Moving to correct directory.."
    cd ..
    cd ..
fi
#Check if in bn-api folder before proceeding
path=$(pwd)
primary_dir=$(basename $path)
if [ "$primary_dir" == "bn-api" ]; then
    echo "    + Correct directory"
else
    echo "    + Error: Incorrect directory -> start code_coverage from script, db or bn_api folder!"
    exit 1
fi

echo "Check if grcov installed:"
if [ "$(command -v grcov)" ]
then
    echo "    + Already installed"
else
    echo "    + Installing.."
    cargo install grcov
fi

echo "Check if lcov installed:"
if [ "$(command -v lcov)" ]
then
    echo "    + Already installed"
else
    echo "    + Installing.."
    ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)" < /dev/null 2> /dev/null
    brew install lcov
fi

echo "Clear Build and Report directories:"
if [ -d "$build_dir" ]; then
    rm -rf $build_dir
    echo "    + Build directory removed"
else
    echo "    + Build directory already cleared"
fi
if [ -d "$report_dir" ]; then
    rm -rf $report_dir
    echo "    + Report directory removed"
else
    echo "    + Report directory already cleared"
fi
#Make clean directories for Build and Report
mkdir $build_dir
mkdir $report_dir

echo "Build project.."
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"
cargo +nightly build --verbose $CARGO_OPTIONS --manifest-path=db/Cargo.toml

echo "Perform project Tests.."
cargo +nightly test --verbose $CARGO_OPTIONS --manifest-path=db/Cargo.toml

echo "Acquire all build and test files for coverage check.."
ccov_filename="ccov.zip"
ccov_path="$report_dir$ccov_filename"
zip -0 $ccov_path `find $build_dir \( -name "bigneon_db*.gc*" \) -print`;

echo "Perform grcov code coverage.."
lcov_filename="lcov.info"
lcov_path="$report_dir$lcov_filename"
grcov $ccov_path -s $source_root_dir -t lcov --llvm --branch --ignore-not-existing --ignore-dir "/*" > $lcov_path;

echo "Generate report from code coverage.."
cd $source_root_dir; #Must be in source folder
local_lcov_path="$local_report_dir$lcov_filename"
genhtml -o $local_report_dir --show-details --highlight --legend $local_lcov_path

echo "Launch report in browser.."
index_str="index.html"
open "$local_report_dir$index_str"
