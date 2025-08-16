mod downloader;

use fitsio::FitsFile;
use fitsio::HeaderValue;
use rayon::prelude::*;
use image::{GrayImage, Luma};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_output_dir = "poss_1";
    let red_output_dir = format!("{}/red", base_output_dir);
    let blue_output_dir = format!("{}/blue", base_output_dir);
    std::fs::create_dir_all(&red_output_dir)?;
    std::fs::create_dir_all(&blue_output_dir)?;
    let ram_disk_path = "/dev/shm";

    // --- Download and Process Red Images ---
    for i in 1..=871 { // There are a max of 871 plates in the digitized POSS-1 archives
        let image_id = format!("XE{:03}", i);
        let download_url = format!("https://irsa.ipac.caltech.edu/data/DSS/images/dss1red/dss1red_{}.fits", image_id);
        let temp_file_path = format!("{}/dss1red_{}.fits", ram_disk_path, image_id);

        match downloader::download_file(&download_url, &temp_file_path).await {
            Ok(_) => {
                println!("Processing red image {}...", image_id);
                if let Err(e) = process_fits_file(&temp_file_path, &red_output_dir) {
                    eprintln!("Error processing red image {}: {}", image_id, e);
                }
                // Clean up the file from RAM disk
                if let Err(e) = std::fs::remove_file(&temp_file_path) {
                    eprintln!("Failed to remove temporary file {}: {}", temp_file_path, e);
                }
            }
            Err(e) => eprintln!("Error downloading red image {}: {}", image_id, e),
        }
    }

    // --- Download and Process Blue Images ---
    for i in 1..=871 { // There are a max of 871 plates in the digitized POSS-1 archives
        let image_id = format!("XO{:03}", i);
        let download_url = format!("https://irsa.ipac.caltech.edu/data/DSS/images/dss1blue/dss1blue_{}.fits", image_id);
        let temp_file_path = format!("{}/dss1blue_{}.fits", ram_disk_path, image_id);

        match downloader::download_file(&download_url, &temp_file_path).await {
            Ok(_) => {
                println!("Processing blue image {}...", image_id);
                if let Err(e) = process_fits_file(&temp_file_path, &blue_output_dir) {
                    eprintln!("Error processing blue image {}: {}", image_id, e);
                }
                // Clean up the file from RAM disk
                if let Err(e) = std::fs::remove_file(&temp_file_path) {
                    eprintln!("Failed to remove temporary file {}: {}", temp_file_path, e);
                }
            }
            Err(e) => eprintln!("Error downloading blue image {}: {}", image_id, e),
        }
    }

    Ok(())
}

fn process_fits_file(fits_path: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut fptr = FitsFile::open(fits_path)?;

    // Extract header info for filenames
    let plate_id_hv: HeaderValue<String> = fptr.hdu(0)?.read_key(&mut fptr, "PLATEID")?;
    let plate_id = &plate_id_hv.value;
    let region_hv: HeaderValue<String> = fptr.hdu(0)?.read_key(&mut fptr, "REGION")?;
    let region = &region_hv.value;

    // Construct dynamic filenames
    let csv_filename = format!("{}/{}_{}_header_values.csv", output_dir, plate_id, region);
    let png_filename = format!("{}/{}_{}.png", output_dir, plate_id, region);

    // Save header to CSV
    let keys = vec![
        "DATE-OBS", "REGION", "PLATEID", "PLATERA", "PLATEDEC", "PLTSCALE", "PLTSIZEX", "PLTSIZEY", "NAXIS1", "NAXIS2",
    ];
    let mut wtr = csv::Writer::from_path(&csv_filename)?;
    wtr.write_record(&["Key", "Value", "Comment"])?;
    for key in keys {
        let header_value: HeaderValue<String> = fptr.hdu(0)?.read_key(&mut fptr, key)?;
        let HeaderValue { value, comment } = header_value;
        wtr.write_record(&[key, &value, comment.as_deref().unwrap_or("")])?;
    }
    wtr.flush()?;
    println!("Header data saved to {}", csv_filename);

    // Save image as PNG
    let hdu = fptr.primary_hdu()?;
    let image_data: Vec<i16> = hdu.read_image(&mut fptr)?;

    let img_x: HeaderValue<String> = fptr.hdu(0)?.read_key(&mut fptr, "NAXIS1")?;
    let HeaderValue { value, .. } = img_x;
    let width = value.parse::<u32>()?;

    let img_y: HeaderValue<String> = fptr.hdu(0)?.read_key(&mut fptr, "NAXIS2")?;
    let HeaderValue { value, .. } = img_y;
    let height = value.parse::<u32>()?;

    let min_value = *image_data.iter().min().unwrap_or(&0);
    let max_value = *image_data.iter().max().unwrap_or(&0);

    let mut img = GrayImage::new(width, height);
    
    let normalized_values: Vec<u8> = image_data
        .par_iter()
        .map(|&value| {
            if max_value == min_value {
                0
            } else {
                ((value as f32 - min_value as f32) / (max_value as f32 - min_value as f32) * 255.0).round() as u8
            }
        })
        .collect();
    for (i, &normalized_value) in normalized_values.iter().enumerate() {
        img.put_pixel((i as u32) % width, (i as u32) / width, Luma([normalized_value]));
    }
    img.save(&png_filename)?;
    println!("Image saved to {}", png_filename);
    
    println!("Processed FITS file for Plate ID: {} and Region: {}", plate_id, region);

    Ok(())
}
