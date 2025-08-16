# Preamble

I've made heavy use of AI coding for this. I've tested that it works but do be warned.

This also does require a Linux environment as I save fits to a ram disk at /dev/shm. This prevents writing over 900 gb of fits to my ssd.

Additionally, this is not fast. Caltech may even block me at some point since I'm scraping so much data. I've yet to download everything. I'll post an update with the final file size if it finishes.

# Code Function

This scrapes the images of the plates available from Caltech and converts them to a normalized, 8 bit png file. There is a bit of dataloss doing that but it makes the file SIGNIFICANTLY smaller. Worth it for what I plan to do.

This does save portions of the fits header file but not everything. Just the stuff I needed.
Add keys to line 77 if you need a specific header value saved.

## Project Structure

The application is split into two main components:

-   `src/main.rs`: The main executable that orchestrates the downloading and processing of images.
-   `src/downloader.rs`: A module that provides a reusable function to download files from a URL with progress indication.

## How to Run

1.  **Prerequisites**: Ensure you have Rust and Cargo installed.
2.  **Build**: Navigate to the project root and run `cargo build --release`.
3.  **Execute**: Run the compiled binary with `cargo run`.

The application will create a `poss_1` directory in the project root containing the processed images and header data.