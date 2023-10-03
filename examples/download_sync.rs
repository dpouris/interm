use interm::{interactive::Line as InteractiveLine, Block};
use std::io::{Error, Result as IoResult};
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Mutex as TokioMutex, time::sleep};

fn generate_interactive_elements<const T: usize>() -> Vec<InteractiveLine> {
    let mut elements: Vec<InteractiveLine> = Vec::with_capacity(T);
    let names = generate_names::<T>();
    for name in names.iter() {
        elements.push(InteractiveLine::new(name));
    }
    elements
}

fn generate_names<const T: usize>() -> Vec<String> {
    let mut names = Vec::with_capacity(T);

    for idx in 0..T {
        names.push(format!("Download {idx}"));
    }

    names
}

async fn download(cur: Arc<TokioMutex<Block>>, elem: InteractiveLine) -> IoResult<()> {
    let r = rand::random::<u8>();

    for i in 0..=100 {
        let progress = (i as f64) / 100.0;
        let mut progress_bar = "=".repeat((progress * 49.0) as usize);
        progress_bar.push('>');
        let content = format!(
            "{prev_content}: [{progress_bar:<50}] {progress:.1}%",
            prev_content = elem.content,
            progress = progress * 100.0,
        );

        // we get a lock on the cursor and update the element, but the lock isn't released until the end of the scope of the loop iteration
        // therefore, we can't update the element again until the next iteration.
        // The problem is that we sleep for x milliseconds, so that means that other threads can't update their elements until we wake up
        // therefore, we need to release the lock before the sleep, but we can't do that until the end of the loop iteration
        // so we need to update the element in its own scope, therefore releasing the lock before the sleep
        // giving the ability on the other threads to update their elements
        {
            cur.lock().await.update_element(&elem, &content, true)?;
        }
        sleep(Duration::from_millis(100 * (r / 100) as u64)).await;
    }

    cur.lock().await.update_element(
        &elem,
        format!(
            "\x1b[34m{elem_name}: Complete\x1b[0m",
            elem_name = elem.content
        )
        .as_str(),
        true,
    )?;

    Ok(())
}

async fn try_main() -> Result<(), Error> {
    let elements = generate_interactive_elements::<10>();
    let cursor = Arc::new(TokioMutex::new(Block::new(elements)?));
    let mut threads = Vec::with_capacity(cursor.lock().await.interactive_lines.len());

    cursor.lock().await.hide_cursor()?;
    for elem in cursor.lock().await.interactive_lines.iter() {
        let cur = Arc::clone(&cursor);
        let elem = elem.clone();
        threads.push(tokio::spawn(download(cur, elem)));
    }

    for thread in threads {
        thread.await??;
    }

    cursor.lock().await.clear_lines()?;
    cursor.lock().await.show_cursor()?;
    println!("\x1b[36mAll downloads complete!\x1b[0m");
    Ok(())
}
#[tokio::main]
async fn main() {
    match try_main().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
