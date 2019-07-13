use std::fs::File;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Read;
use walkdir::WalkDir;
use std::path::Path;
use std::ffi::OsStr;
use std::collections::HashSet;
use std::{fs, env};

#[derive(Debug)]
enum PixelFormat {
    Greyscale,
    TrueColor,
    IndexedColor,
    GreyscaleWithAlpha,
    TrueColorWithAlpha,
}

#[derive(Debug)]
enum ParseResult {
    OpenFail,
    ReadFail,
    InvalidPngHeader,
    InvalidIhdr,
    InvalidPixelFormat,
    Valid(PixelFormat),
}

const EXPECTED_PNG_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

//check PNG header ( 137 80 78 71 13 10 26 10)
//check IHDR size (4 bytes, big endian)
//cheeck IHDR type ("IHDR" string)
//width 4 bytes
//height 4 bytes
//bit depth 1 byte
//color type 1 byte <- pixel format
//- greyscale = 0
//- truecolor = 2
//- indexed-color = 3
//- greyscale with alpha = 4
//- truecolor with alpha = 6
//filter method 1 byte
//interlace method 1 byte
fn parse_one(filename : &Path) -> ParseResult {
    let ihdr_expected: &[u8] = "IHDR".as_bytes();

    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(_e) => return ParseResult::OpenFail,
    };

    //Check png header
    let mut png_header : [u8; 8] = [0; 8];
    match file.read_exact(&mut png_header) {
        Ok(()) => {
            if png_header != EXPECTED_PNG_HEADER {
                return ParseResult::InvalidPngHeader;
            }
        },
        Err(_e) => return ParseResult::ReadFail,
    }

    //Get ihdr size
    let _ihdr_size : u32 = match file.read_u32::<BigEndian>() {
        Ok(size) => size,
        Err(_e) => return ParseResult::ReadFail,
    };

    // Check IHDR
    let mut ihdr : [u8; 4] = [0; 4];
    match file.read_exact(&mut ihdr) {
        Ok(()) => {
           if ihdr != ihdr_expected {
               return ParseResult::InvalidIhdr;
           }
        },
        Err(_e) => return ParseResult::ReadFail,
    };

    //read image width
    if file.read_u32::<BigEndian>().is_err() {
        return ParseResult::ReadFail;
    }


    //read image height
    if file.read_u32::<BigEndian>().is_err() {
        return ParseResult::ReadFail;
    }

    //read bit depth
    if file.read_u8().is_err() {
        return ParseResult::ReadFail;
    }

    //read pixel format
    return match file.read_u8() {
        Ok(pixel_format_byte) => {
            match pixel_format_byte {
                0 => ParseResult::Valid(PixelFormat::Greyscale),
                2 => ParseResult::Valid(PixelFormat::TrueColor),
                3 => ParseResult::Valid(PixelFormat::IndexedColor),
                4 => ParseResult::Valid(PixelFormat::GreyscaleWithAlpha),
                6 => ParseResult::Valid(PixelFormat::TrueColorWithAlpha),
                _ => ParseResult::InvalidPixelFormat,
            }
        }
        Err(_e) => ParseResult::ReadFail,
    }
}

// Convert an image to RGB/RGBA format, then optimize it
fn fix_image(path : &Path) {
    let image_size_before = fs::metadata(path).expect("Can't get image size").len() as f32;

    print!("Converting to RGB/RGBA...");
    let image_before_optimizing = image::open(path).expect("Failed to open image!");


    //image "0.21.2" will save as RGBA32 format
    image_before_optimizing.save(path).expect("Failed to save image!");

    print!(" Optimizing...");

    let inpath = oxipng::InFile::Path(path.to_path_buf());
    let outpath = oxipng::OutFile::Path(None);

    oxipng::optimize(&inpath,
                     &outpath,
                     &oxipng::Options {
                        alphas: HashSet::new(), //Disable Alpha optimizations
                         color_type_reduction: false,
                         ..Default::default()
                     })
        .expect("Optimize failed!");

    print!("Optimized.");
    println!();

    let image_pixel_data_after_optimizing = image::open(path)
        .expect("Failed to open optimized image!")
        .raw_pixels();

    // Check the images are 100% identical
    if image_before_optimizing.raw_pixels() != image_pixel_data_after_optimizing {
        println!("---------------------------------------------");
        println!("ERROR: optimized image wasn't identical to original image");
        println!("---------------------------------------------");
        std::process::exit(-1);
    }


    let image_size_after = fs::metadata(path).expect("Can't get image size").len() as f32;
    println!("-------------------------------");
    println!("Size Change: [{:+}KB / {:3.0}%]",
             (image_size_after - image_size_before) / 1000f32,
             image_size_after / image_size_before * 100f32);
    println!("-------------------------------");
}

fn handle_one_file(path : &Path, rel_path : &Path) -> bool {
    return match parse_one(path) {
        ParseResult::Valid(pixel_format) => {
            match pixel_format {
                PixelFormat::IndexedColor => {
                    println!("{} is indexed!", rel_path.display());
                    fix_image(path);
                    true
                }
                _ => false,
            }
        },

        error_parse_result => {
            println!("Error {:?}: {}", error_parse_result, rel_path.display());
            false
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("First argument must be path to folder to be processed.");
        return;
    }

    let target_extension = OsStr::new("png");
    let scan_path = Path::new(&args[1]);

    println!("Scanning [{}]", scan_path.display());

    let mut num_fixed = 0;
    for entry in WalkDir::new(scan_path) {
        let entry = entry.expect("File I/O Error?");
        let path = entry.path();
        let rel_path = path.strip_prefix(scan_path).unwrap();

        // Skip non-files
        if !path.is_file() {
            continue;
        }

        // Only process files with .png extension
        match path.extension() {
            Some(ext) => {
                if ext == target_extension {
                    if handle_one_file(path, rel_path) {
                        num_fixed += 1;
                    }
                }
            },
            None => continue,
        }
    }

    println!("Fixed {} files.", num_fixed);
}
