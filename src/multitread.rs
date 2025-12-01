use std::fs::File;
use std::io::{self, Read, Write};
use std::thread;
use reqwest::blocking::Client;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

// Parallel download function
pub fn download_parallel(
    client: &Client,
    url: &str,
    filename: &str,
    total_size: u64,
    num_threads: usize,
) -> io::Result<()>
{
    // Create multi-progress instance to manage multiple progress bars
    let mp = MultiProgress::new();
    
    // Calculate chunk size
    let chunk_size = (total_size + num_threads as u64 - 1) / num_threads as u64;
    
    // Create threads and download chunks
    let mut handles = vec![];
    
    for i in 0..num_threads {
        let client = client.clone();
        let url = url.to_string();
        let start = i as u64 * chunk_size;
        let end = std::cmp::min(start + chunk_size - 1, total_size - 1);
        let chunk_length = end - start + 1;
        
        // Create individual progress bar for each thread
        let pb = mp.add(ProgressBar::new(chunk_length));
        let template = format!("Thread {}: {{spinner:.green}} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{bytes_per_sec}}, {{eta}})", i+1);
        pb.set_style(ProgressStyle::with_template(&template)
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "));
        pb.set_message(format!("Downloading chunk {}-{}", start, end));
        
        handles.push(thread::spawn(move || {
            let mut chunk = Vec::new();
            let range_header = format!("bytes={}-{}", start, end);
            
            let mut response = client.get(&url)
                .header("User-Agent", "egit-cli")
                .header("Range", range_header)
                .send()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            
            // Read response in chunks to update progress
            let mut buffer = [0; 8192];
            loop {
                match response.read(&mut buffer) {
                    Ok(0) => break, // End of file
                    Ok(n) => {
                        chunk.extend_from_slice(&buffer[..n]);
                        pb.inc(n as u64);
                    },
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::Other, e));
                    }
                }
            }
            
            pb.finish_with_message(format!("Chunk {}-{} completed", start, end));
            Ok(chunk)
        }));
    }
    
    // Wait for all threads to complete and collect chunks
    let mut results = vec![];
    for handle in handles {
        let result = handle.join().unwrap()?;
        results.push(result);
    }
    
    // Write all chunks to file in order
    let mut file = File::create(filename)?;
    for chunk in results {
        file.write_all(&chunk)?;
    }
    
    Ok(())
}
