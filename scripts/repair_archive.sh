#!/bin/bash
7za x -aoa *.7z -omy_temp_extract_dir || exit 1
./png_header_scanner my_temp_extract_dir || exit 1
7za a temp_result.7z ./my_temp_extract_dir/* -mx9 || exit 1
rm -rf my_temp_extract_dir || exit 1
