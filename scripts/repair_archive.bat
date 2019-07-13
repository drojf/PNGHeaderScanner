7za x -aoa *.7z -otemp_extract_dir
png_header_scanner temp_extract_dir
7za a temp_result.7z ./temp_extract_dir/* -mx9 
pause