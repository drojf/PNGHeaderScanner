SETLOCAL EnableDelayedExpansion

7za x -aoa *.7z -otemp_extract_dir || exit /b !ERRORLEVEL!
png_header_scanner temp_extract_dir || exit /b !ERRORLEVEL!
7za a temp_result.7z ./temp_extract_dir/* -mx9 || exit /b !ERRORLEVEL!
